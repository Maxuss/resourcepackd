use notify::{event::CreateKind, Config, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{comp::compile_pack, WatchCmd};

pub fn watch(cmd: WatchCmd) -> anyhow::Result<()> {
    tracing::info!("Watching directory {:?} for changes", cmd.root_dir);

    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(&cmd.root_dir, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Access(_)
                | notify::EventKind::Create(CreateKind::File)
                | notify::EventKind::Remove(_) => {
                    tracing::info!("Detected changes, recompiling...");
                    if let Err(error) = compile_pack(cmd.root_dir.clone(), cmd.out_file.clone()) {
                        tracing::error!("Failed to compile resource pack: {error}")
                    }
                }
                _ => {}
            },
            Err(e) => {
                tracing::error!("Watching error: {:?}", e);
                return Err(anyhow::Error::from(e));
            }
        }
    }

    Ok(())
}
