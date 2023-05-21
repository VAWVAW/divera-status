#![cfg(feature = "dbus-interface")]
use dbus as dbus;
#[allow(unused_imports)]
use dbus::arg;
use dbus_crossroads as crossroads;

pub trait DeNlihDiveraStatus1Monitor {
    fn update(&mut self) -> Result<(), dbus::MethodErr>;
}

pub fn register_de_nlih_divera_status1_monitor<T>(cr: &mut crossroads::Crossroads) -> crossroads::IfaceToken<T>
where T: DeNlihDiveraStatus1Monitor + Send + 'static
{
    cr.register("de.nlih.DiveraStatus1.Monitor", |b| {
        b.method("Update", (), (), |_, t: &mut T, ()| {
            t.update()
        });
    })
}

pub trait DeNlihDiveraStatus1Status {
    fn next(&mut self) -> Result<(), dbus::MethodErr>;
    fn previous(&mut self) -> Result<(), dbus::MethodErr>;
}

pub fn register_de_nlih_divera_status1_status<T>(cr: &mut crossroads::Crossroads) -> crossroads::IfaceToken<T>
where T: DeNlihDiveraStatus1Status + Send + 'static
{
    cr.register("de.nlih.DiveraStatus1.Status", |b| {
        b.method("Next", (), (), |_, t: &mut T, ()| {
            t.next()
        });
        b.method("Previous", (), (), |_, t: &mut T, ()| {
            t.previous()
        });
    })
}
