//! Atomic last-run evidence. The OS releases ownership even after forced exit.
use std::{fs::{self, File, OpenOptions}, io::{self, Read, Write}, path::{Path, PathBuf}};
use crate::app_lifecycle::RunId;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RunRecord {
    #[serde(flatten)] pub identity: RunId,
    pub outcome: String,
    pub quit_requested: bool,
    pub flush: String,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PreviousRun { None, Clean, InterruptedUnknown }
impl PreviousRun {
    pub fn as_str(self) -> &'static str {
        match self { Self::None => "previous_absent", Self::Clean => "previous_clean_exit",
            Self::InterruptedUnknown => "previous_interrupted_unknown" }
    }
}
pub struct RunStore { directory: PathBuf, _ownership: File }
impl RunStore {
    pub fn claim(directory: &Path) -> io::Result<(Self, PreviousRun)> {
        fs::create_dir_all(directory)?;
        let ownership = OpenOptions::new().read(true).write(true).create(true)
            .truncate(false).open(directory.join("lifecycle.owner"))?;
        ownership.try_lock().map_err(io::Error::from)?;
        // No previous-run state read or replacement before exclusive ownership.
        let previous = match File::open(directory.join("lifecycle.json")) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => PreviousRun::None,
            Ok(file) => {
                let mut bytes = Vec::new();
                let record = file.take(8193).read_to_end(&mut bytes).ok()
                    .filter(|_| bytes.len() <= 8192)
                    .and_then(|_| serde_json::from_slice::<RunRecord>(&bytes).ok());
                if record.is_some_and(|r| r.outcome == "clean_exit") { PreviousRun::Clean }
                else { PreviousRun::InterruptedUnknown }
            }
            Err(_) => PreviousRun::InterruptedUnknown,
        };
        Ok((Self { directory: directory.into(), _ownership: ownership }, previous))
    }
    pub fn write(&self, record: &RunRecord) -> io::Result<()> {
        let temporary = self.directory.join("lifecycle.json.tmp");
        let mut file = File::create(&temporary)?;
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        // Same-directory atomic replacement; never delete the old record first.
        fs::rename(temporary, self.directory.join("lifecycle.json"))
    }
}
