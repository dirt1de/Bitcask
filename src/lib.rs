// 1. Write first or compact first?
// 2. The active data file is also part of the reader files
// 3. Remove command itself can be deleted (together with the previous set command)
// 4. Too many files being opened mutliplt times => slows down the db

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};
use thiserror::Error;

const COMPACT_THRESHOLD: usize = 24;

pub type Result<T> = std::result::Result<T, KvError>;

#[derive(Error, Debug)]
pub enum KvError {
    #[error("Encountered IO error `{0}`")]
    Io(#[from] io::Error),
    #[error("Key to be removed: `{0}` is not found")]
    KeyNotFound(String),
    #[error("Serde parsing error `{0}`")]
    SerdeError(#[from] serde_json::Error),
}

pub struct KvStore {
    file_id: usize,                // The file_id of the current active data file for write
    writer: File,                  // the file handle for the active data file
    readers: HashMap<usize, File>, // file_id -> reader of that file
    dir: PathBuf,
    key_dir: BTreeMap<String, KeyDirValue>,
    uncompacted: usize,
}

#[derive(Debug)]
pub struct KeyDirValue {
    file_id: usize, // the file_id the value is stored in
    value_size: usize,
    start_index: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Command {
    Set(String, String),
    Remove(String),
}

impl KvStore {
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let key_clone = key.clone();
        let cmd = Command::Set(key, value);

        // Writer first, then compact. Not the other way around

        // FIRST VERSION: After compact, we then write
        // 2. Serialize the command into strings, and record the value_size and start_index
        let mut file = &mut self.writer;

        let cmd_string = serde_json::to_string(&cmd)?;
        let value_size = cmd_string.len();
        let start_index = file.seek(SeekFrom::End(0))?;

        // 3. Write the serialized json into the created file
        // If the write returns an Err, returns it
        file.write(cmd_string.as_bytes())?;

        // 4. If the write is successful, we store the meta information
        // into the in-memory key_dir
        let key_dir_value = KeyDirValue {
            file_id: self.file_id,
            value_size,
            start_index,
        };
        if let Some(key_dir_value) = self.key_dir.insert(key_clone, key_dir_value) {
            self.uncompacted += key_dir_value.value_size;
        };

        // FIRST VERSION: We need to compact first, then write the latest command
        // to the latest writer.
        if self.uncompacted > COMPACT_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        // 1. Get the meta information from the key_dir
        let key_dir_value = self.key_dir.get(&key);

        // If the key exits, we open the file and extract the command
        if let Some(key_dir_value) = key_dir_value {
            let file_id = key_dir_value.file_id;
            let start_index = key_dir_value.start_index;
            let value_size = key_dir_value.value_size;

            // Extract the whole command from the log and deserialize
            let file = self.readers.get_mut(&file_id).unwrap();
            file.seek(SeekFrom::Start(start_index))?;
            let cmd_reader = file.take(value_size as u64);
            let result: Command = serde_json::from_reader(cmd_reader)?;

            if let Command::Set(_k, v) = result {
                return Ok(Some(v));
            } else {
                // this will not execute
                return Ok(None);
            }
        } else {
            return Ok(None);
        }
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        // Remove the key from the in-memory hashmap
        match self.key_dir.remove(&key) {
            Some(value) => {
                let mut file = &mut self.writer;

                let cmd = Command::Remove(key);
                let cmd_string = serde_json::to_string(&cmd)?;
                file.write(cmd_string.as_bytes())?;

                self.uncompacted += cmd_string.len() + value.value_size;

                Ok(())
            }
            None => Err(KvError::KeyNotFound(key)),
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        // 1. Create the directory to the path
        let dir = path.into();
        std::fs::create_dir_all(&dir)?;

        let mut uncompacted = 0;
        let mut file_id = 0;
        // replay_log() should return a writer
        let mut readers = HashMap::new();
        let mut key_dir = BTreeMap::new();

        // 2. when you open the file, you should replay the logs
        // to mutate the fields of KvStore and return the writer
        replay_log(
            &dir,
            &mut file_id,
            &mut readers,
            &mut key_dir,
            &mut uncompacted,
        )?;

        let writer = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&dir, file_id))?;

        let latest_reader = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&dir, file_id))?;

        readers.insert(file_id, latest_reader);

        Ok(KvStore {
            file_id,
            writer,
            readers,
            dir,
            key_dir,
            uncompacted,
        })
    }

    // Need to update file_id, writer, readers, key_dir, uncompacted
    // For write, you need to store the new commmands in the new writer file
    // Thus, you need to change (file_id, writer)

    // In addition, you need to direct all read (GET) to the compact_file
    // Thus, you need to change (readers, key_dir)

    // For readers: need to delete all file_id smaller than file_id from the hashmap.
    // In addition, need to insert the (compact_file_id, compact_file) into readesr

    // For key_dir: need to edit the file_id and start_index for each KeyDirValue
    fn compact(&mut self) -> Result<()> {
        let new_writer_id = self.file_id + 2;
        let compact_file_id = self.file_id + 1;

        // Create the new writer to handle future writes
        let new_writer = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&self.dir, new_writer_id))?;

        // Update self with info of this new writer
        self.writer = new_writer;
        self.file_id = new_writer_id;

        // Create the compact_file and copy all currently active k-v commands
        // from the old readers to compact_file
        let mut compact_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(log_path(&self.dir, compact_file_id))?;

        let mut start_index = 0; // Used to keep track of the start_position for each command
                                 // copied into the compact_file

        // copy the commands from the old reader file to compact_file and
        // update command.file_id and command.start_index
        for command in self.key_dir.values_mut() {
            let reader_id = command.file_id;
            // Here has problem
            let mut reader = self.readers.get_mut(&reader_id).unwrap();
            reader.seek(SeekFrom::Start(command.start_index));

            let mut content = reader.take(command.value_size as u64);

            let value_size = io::copy(&mut content, &mut compact_file)?;

            // Update the key_dir_value inplace
            command.file_id = compact_file_id;
            command.start_index = start_index;

            start_index += value_size;
        }

        // Delete and remove from readers all files with file_id less than compact_file_id;

        let remove_file_ids: Vec<_> = self
            .readers
            .keys()
            .filter(|key| key < &&compact_file_id)
            .cloned()
            .collect();

        for file_id in remove_file_ids {
            self.readers.remove(&file_id);
            fs::remove_file(log_path(&self.dir, file_id))?;
        }

        // Make all future reads to compact_file
        self.readers.insert(compact_file_id, compact_file);

        let mut new_writer_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true)
            .open(log_path(&self.dir, new_writer_id))?;

        self.readers.insert(new_writer_id, new_writer_file);

        self.uncompacted = 0;

        Ok(())
    }
}

fn log_path(dir: &PathBuf, file_id: usize) -> PathBuf {
    dir.join(format!("{}.log", file_id))
}

// The only problem right now is how  to make the  writer to be
// one of the readers
pub fn replay_log(
    dir: &PathBuf,
    file_id: &mut usize,
    readers: &mut HashMap<usize, File>,
    key_dir: &mut BTreeMap<String, KeyDirValue>,
    uncompacted: &mut usize,
) -> Result<()> {
    // need to find all files in the dir.
    // the file with largest file_id will be the writer, and the rest
    // of them will be put into the readers
    let mut file_ids = vec![];

    // Can be optimized (maybe in this loop, we can construct the readers?)
    // Get sorted file name vector
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.file_name().to_str().unwrap().find(".log") != None {
            let file_id = entry
                .file_name()
                .to_str()
                .unwrap()
                .strip_suffix(".log")
                .unwrap()
                .parse::<usize>()
                .unwrap();
            file_ids.push(file_id);
        }
    }

    if file_ids.is_empty() {
        // If there is no log in the directory,
        // It means this is the first start. So you can just create a single writer file
        let writer = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&dir, 0))?;

        readers.insert(0, writer);
        *file_id = 0;

        return Ok(());
    }

    file_ids.sort();

    // Update the writer
    // Every time a kvstore is reopen, create a new writer file
    let writer_file_id = file_ids[file_ids.len() - 1] + 1;

    *file_id = writer_file_id;

    for id in file_ids[0]..=file_ids[file_ids.len() - 1] {
        // Update the reader
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&dir, id))?;

        // Loop each reader file, execute the following lines
        let mut stream = Deserializer::from_reader(file).into_iter::<Command>();

        let mut index = 0;

        while let Some(result) = stream.next() {
            let cmd = result?;
            let value_size = stream.byte_offset() - index;

            let cmd_clone = cmd.clone();

            match cmd {
                Command::Set(k, _v) => {
                    let start_index = index;
                    // we can get the length of the value in the disk
                    // by using stream.byte_offset()
                    let key_dir_value = KeyDirValue {
                        file_id: id,
                        start_index: start_index as u64,
                        value_size,
                    };

                    if let Some(key_dir_value) = key_dir.insert(k, key_dir_value) {
                        *uncompacted += key_dir_value.value_size;
                    }
                }
                Command::Remove(k) => {
                    if let Some(key_dir_value) = key_dir.remove(&k) {
                        *uncompacted += key_dir_value.value_size;

                        let cmd_string = serde_json::to_string(&cmd_clone)?;
                        let value_size = cmd_string.len();
                        *uncompacted += value_size;
                    }
                }
            }
            index += value_size;
        }

        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .append(true)
            .create(true)
            .open(log_path(&dir, id))?;
        readers.insert(id, file);
    }

    return Ok(());
}
