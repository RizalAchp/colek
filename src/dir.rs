#![allow(unused)]

use futures::{future::BoxFuture, stream, Future, FutureExt, Stream, StreamExt};
use std::{
    path::{Path, PathBuf},
    pin::Pin,
    task::Poll,
};
use tokio::{
    fs::{self, read_dir, DirEntry, ReadDir},
    io,
};

use crate::filters::FilterFn;

type BoxStream = futures::stream::BoxStream<'static, io::Result<DirEntry>>;
pub struct WalkDir {
    root: PathBuf,
    entries: BoxStream,
}

enum State {
    Start(PathBuf),
    Walk(Vec<ReadDir>),
    Done,
}

type UnfoldState = (io::Result<DirEntry>, State);

impl WalkDir {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_owned(),
            entries: walk_dir(root),
        }
    }
}

impl Stream for WalkDir {
    type Item = io::Result<DirEntry>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let entries = Pin::new(&mut self.entries);
        entries.poll_next(cx)
    }
}

fn walk_dir(root: impl AsRef<Path>) -> BoxStream {
    stream::unfold(
        State::Start(root.as_ref().to_owned()),
        move |state| async move {
            match state {
                State::Start(root) => match read_dir(root).await {
                    Err(e) => Some((Err(e), State::Done)),
                    Ok(rd) => walk(vec![rd]).await,
                },
                State::Walk(dirs) => walk(dirs).await,
                State::Done => None,
            }
        },
    )
    .boxed()
}

fn walk(mut dirs: Vec<ReadDir>) -> BoxFuture<'static, Option<UnfoldState>> {
    async move {
        let Some(dir) = dirs.last_mut() else {
            return None;
        };
        match dir.next_entry().await {
            Err(e) => Some((Err(e), State::Walk(dirs))),
            Ok(None) => {
                dirs.pop();
                walk(dirs).await
            }
            Ok(Some(entry)) => match entry.file_type().await {
                Err(e) => Some((Err(e), State::Walk(dirs))),
                Ok(ft) => {
                    if ft.is_dir() {
                        let rd = match read_dir(entry.path()).await {
                            Err(e) => return Some((Err(e), State::Walk(dirs))),
                            Ok(rd) => rd,
                        };
                        dirs.push(rd);
                    }
                    Some((Ok(entry), State::Walk(dirs)))
                }
            },
        }
    }
    .boxed()
}
