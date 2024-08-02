//! # D-Bus interface proxy for: `org.kde.StatusNotifierItem`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `dbus_status_notifier_item.xml`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;
#[proxy(interface = "org.kde.StatusNotifierItem", assume_defaults = true)]
trait StatusNotifierItem {
    /// Activate method
    fn activate(&self, x: i32, y: i32) -> zbus::Result<()>;

    /// ContextMenu method
    fn context_menu(&self, x: i32, y: i32) -> zbus::Result<()>;

    /// Scroll method
    fn scroll(&self, delta: i32, orientation: &str) -> zbus::Result<()>;

    /// SecondaryActivate method
    fn secondary_activate(&self, x: i32, y: i32) -> zbus::Result<()>;

    /// NewAttentionIcon signal
    #[zbus(signal)]
    fn new_attention_icon(&self) -> zbus::Result<()>;

    /// NewIcon signal
    #[zbus(signal)]
    fn new_icon(&self) -> zbus::Result<()>;

    /// NewOverlayIcon signal
    #[zbus(signal)]
    fn new_overlay_icon(&self) -> zbus::Result<()>;

    /// NewStatus signal
    #[zbus(signal)]
    fn new_status(&self, status: &str) -> zbus::Result<()>;

    /// NewTitle signal
    #[zbus(signal)]
    fn new_title(&self) -> zbus::Result<()>;

    /// NewToolTip signal
    #[zbus(signal)]
    fn new_tool_tip(&self) -> zbus::Result<()>;

    /// AttentionIconName property
    #[zbus(property)]
    fn attention_icon_name(&self) -> zbus::Result<String>;

    /// AttentionIconPixmap property
    #[zbus(property)]
    fn attention_icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;

    /// AttentionMovieName property
    #[zbus(property)]
    fn attention_movie_name(&self) -> zbus::Result<String>;

    /// Category property
    #[zbus(property)]
    fn category(&self) -> zbus::Result<String>;

    /// IconName property
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    /// IconPixmap property
    #[zbus(property)]
    fn icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;

    /// IconThemePath property
    #[zbus(property)]
    fn icon_theme_path(&self) -> zbus::Result<String>;

    /// Id property
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    /// ItemIsMenu property
    #[zbus(property)]
    fn item_is_menu(&self) -> zbus::Result<bool>;

    /// Menu property
    #[zbus(property)]
    fn menu(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// OverlayIconName property
    #[zbus(property)]
    fn overlay_icon_name(&self) -> zbus::Result<String>;

    /// OverlayIconPixmap property
    #[zbus(property)]
    fn overlay_icon_pixmap(&self) -> zbus::Result<Vec<(i32, i32, Vec<u8>)>>;

    /// Status property
    #[zbus(property)]
    fn status(&self) -> zbus::Result<String>;

    /// Title property
    #[zbus(property)]
    fn title(&self) -> zbus::Result<String>;

    /// ToolTip property
    #[zbus(property)]
    #[allow(clippy::type_complexity)]
    fn tool_tip(&self) -> zbus::Result<(String, Vec<(i32, i32, Vec<u8>)>, String, String)>;
}
