mod dbus_status_notifier_item;
pub use dbus_status_notifier_item::StatusNotifierItemProxy;

mod dbus_status_notifier_watcher;
pub use dbus_status_notifier_watcher::{
    StatusNotifierItemRegistered, StatusNotifierItemUnregistered, StatusNotifierWatcherProxy,
};
