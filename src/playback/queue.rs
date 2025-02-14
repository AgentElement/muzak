use std::{fmt::Display, sync::Arc};

use gpui::{App, AppContext, Entity, RenderImage, SharedString};

use crate::{data::interface::GPUIDataInterface, library::db::LibraryAccess, ui::models::Models};

#[derive(Clone, Debug, PartialEq)]
pub struct QueueItemData {
    data: Entity<Option<QueueItemUIData>>,
    db_id: Option<i64>,
    db_album_id: Option<i64>,
    path: String,
}

impl Display for QueueItemData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.path)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueueItemUIData {
    pub image: Option<Arc<RenderImage>>,
    pub name: Option<SharedString>,
    pub artist_name: Option<SharedString>,
    pub source: DataSource,
}

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum DataSource {
    Metadata,
    Library,
}

impl QueueItemData {
    pub fn new(cx: &mut App, path: String, db_id: Option<i64>, db_album_id: Option<i64>) -> Self {
        QueueItemData {
            path,
            db_id,
            db_album_id,
            data: cx.new(|_| None),
        }
    }

    pub fn get_data(&self, cx: &mut App) -> Entity<Option<QueueItemUIData>> {
        let model = self.data.clone();
        let track_id = self.db_id;
        let album_id = self.db_album_id;
        let path = self.path.clone();
        model.update(cx, move |m, cx| {
            if m.is_some() {
                return;
            }
            *m = Some(QueueItemUIData {
                image: None,
                name: None,
                artist_name: None,
                source: DataSource::Library,
            });

            if let (Some(track_id), Some(album_id)) = (track_id, album_id) {
                let album =
                    cx.get_album_by_id(album_id, crate::library::db::AlbumMethod::Thumbnail);
                let track = cx.get_track_by_id(track_id);

                if let (Ok(track), Ok(album)) = (track, album) {
                    m.as_mut().unwrap().name = Some(track.title.clone().into());
                    m.as_mut().unwrap().image = album.thumb.clone().map(|v| v.0);

                    if let Ok(artist) = cx.get_artist_by_id(album.artist_id) {
                        m.as_mut().unwrap().artist_name = artist.name.clone().map(|v| v.into());
                    }
                }

                cx.notify();
            }

            if m.as_ref().unwrap().artist_name.is_some() {
                return;
            }
            // vital information left blank, try retriving the metadata from disk
            // much slower, especially on windows
            let queue_model = cx.global::<Models>().queue.clone();
            let path_clone = path.clone();

            cx.subscribe(
                &queue_model,
                move |m, _, ev: &(String, QueueItemUIData), cx| {
                    if ev.0 == path_clone {
                        *m = Some(ev.1.clone());
                    }
                    cx.notify();
                },
            )
            .detach();

            cx.global::<GPUIDataInterface>().get_metadata(path);
        });

        model
    }

    pub fn drop_data(&self, cx: &mut App) {
        self.data.update(cx, |m, cx| {
            *m = None;
            cx.notify();
        });
    }

    pub fn get_path(&self) -> &String {
        &self.path
    }
}
