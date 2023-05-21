use crate::WatchCmd;

pub fn watch(cmd: WatchCmd) {
    tracing::info!("Watching directory {:?} for changes", cmd.root_dir);
}
