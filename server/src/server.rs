use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread::{Builder, JoinHandle},
    time::{Duration, Instant},
};
use crossbeam_channel::{tick, unbounded, Receiver, RecvError, Sender};
use log::*;
use once_cell::sync::OnceCell;
use serde_derive::Deserialize;
use thiserror::Error;
use tokio::sync::mpsc::{channel, error::TrySendError as TokioTrySendError, Sender as TokioSender};
use tonic::{metadata::MetadataValue, Request, Response, Status};
use uuid::Uuid;




enum SubscriptionAddedEvent{
    EntryUpdateSubscription{
        uuid: Uuid,
        notification_sender: NotificationSender
    }
}


