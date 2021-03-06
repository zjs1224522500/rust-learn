use std::collections::HashMap;
use std::path::{PathBuf, Path};
use crate::{Result, KvsError};
use serde::{Serialize, Deserialize};
use std::io::{Seek, Read, BufReader, SeekFrom, Write, BufWriter};
use std::{io, fs};
use std::fs::{File, OpenOptions};
use serde_json::Deserializer;
use std::ffi::OsStr;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

/// The `KvStore` stores string key/value pairs.
pub struct KvStore {
    path: PathBuf,
    readers: HashMap<u64, BufReaderWithPos<File>>,
    writer: BufWriterWithPos<File>,
    index: HashMap<String, CommandPos>,
    current_gen: u64,
    //number of bytes than can be saved after a compaction.
    uncompacted: u64,
}

impl KvStore {
    /// Open a `KvStore` with the given path.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        let mut readers = HashMap::<u64, BufReaderWithPos<File>>::new();
        let mut index = HashMap::<String, CommandPos>::new();

        let gen_list = sorted_gen_list(&path)?;
        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut reader, &mut index)?;
            readers.insert(gen, reader);
        }
        let current_gen = gen_list.last().unwrap_or(&0) + 1;


        let writer = new_log_file(&path, current_gen, &mut readers)?;

        Ok(KvStore {
            path,
            readers,
            writer,
            index,
            current_gen,
            uncompacted
        })
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, .. } = cmd {
            let cmd_pos = CommandPos {gen: self.current_gen, pos, len: self.writer.pos - pos };
            if let Some(old_cmd) = self.index.insert(key, cmd_pos) {
                self.uncompacted += old_cmd.len;
            }
            if self.uncompacted > COMPACTION_THRESHOLD {

            }
        }
        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?
        }

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if let Some(cmd_pos) = self.index.get(&key) {
            let reader = self.readers.get_mut(&cmd_pos.gen)
                .expect(format!("Can't find reader: {}", &cmd_pos.gen).as_str());
            reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            let cmd_reader = reader.take(cmd_pos.len);
            if let Command::Set { value, .. } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvsError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                self.index.remove(&key).expect("key not found");
            }
            Ok(())
        } else {
            Err(KvsError::KeyNotFound)
        }
    }

    pub fn compact(&mut self) -> Result<()> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = self.new_log_file(self.current_gen)?;

        let mut new_pos = 0;
        let mut compaction_writer = self.new_log_file(compaction_gen)?;
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self.readers.get_mut(&cmd_pos.gen)
                .expect(format!("Can't find reader: {}", &cmd_pos.gen).as_str());
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut cmd_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut cmd_reader, &mut compaction_writer)?;
            *cmd_pos = CommandPos {gen: compaction_gen, pos: new_pos, len };
            new_pos += len;
        }
        compaction_writer.flush()?;

        //remove stale log files.
        let stale_gens: Vec<_> = self.readers.keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned().collect();
        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }
        self.uncompacted = 0;

        Ok(())
    }

    fn new_log_file(&mut self, gen: u64) -> Result<BufWriterWithPos<File>> {
        new_log_file(&self.path, gen, &mut self.readers)
    }


}

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

struct BufReaderWithPos<R: Read + Seek> {
    reader: BufReader<R>,
    pos: u64,
}

impl<R: Read + Seek> BufReaderWithPos<R> {
    fn new(mut inner: R) -> Result<Self> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(BufReaderWithPos {
            reader: BufReader::new(inner),
            pos,
        })
    }
}

impl<R: Read + Seek> Read for BufReaderWithPos<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}

impl<R: Read + Seek> Seek for BufReaderWithPos<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<W: Write + Seek> {
    writer: BufWriter<W>,
    pos: u64,
}

impl<W: Write + Seek> BufWriterWithPos<W> {
    fn new(mut inner: W) -> Result<Self> {
        let pos = inner.seek(SeekFrom::End(0))?;
        Ok(BufWriterWithPos {
            writer: BufWriter::new(inner),
            pos,
        })
    }
}

impl<W: Write + Seek> Write for BufWriterWithPos<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<W: Write + Seek> Seek for BufWriterWithPos<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

#[derive(Debug)]
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

fn load(gen: u64, reader: &mut BufReaderWithPos<File>, index: &mut HashMap<String, CommandPos>) -> Result<u64> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    // number of bytes than can be saved after a compaction.
    let mut uncompacted = 0;
    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, .. } => {
                if let Some(old_cmd) = index.insert(key, CommandPos {gen, pos, len: new_pos - pos }) {
                    uncompacted += old_cmd.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos;
    }
    Ok(uncompacted)
}

/// return sorted generation numbers in the given directory
fn sorted_gen_list(path: &Path) -> Result<Vec<u64>> {
    let mut gen_list: Vec<u64> = fs::read_dir(path)?
        .flat_map(|res| -> Result<_> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten().collect();
    gen_list.sort_unstable();
    Ok(gen_list)
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn new_log_file(path: &Path, gen: u64, readers: &mut HashMap<u64, BufReaderWithPos<File>>) -> Result<BufWriterWithPos<File>> {
    let path = log_path(path, gen);
    let writer = BufWriterWithPos::new(OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&path)?)?;
    readers.insert(gen, BufReaderWithPos::new(File::open(&path)?)?);
    Ok(writer)
}

// fn print_index(map: &HashMap<String, CommandPos>) {
//     println!("print index");
//     for (key, value) in map.iter() {
//         println!("{}, {:?}", key, value);
//     }
// }