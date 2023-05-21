use dbus::blocking;

pub trait DeNlihDiveraStatus1Monitor {
    fn update(&self) -> Result<(), dbus::Error>;
}

impl<'a, T: blocking::BlockingSender, C: std::ops::Deref<Target = T>> DeNlihDiveraStatus1Monitor
    for blocking::Proxy<'a, C>
{
    fn update(&self) -> Result<(), dbus::Error> {
        self.method_call("de.nlih.DiveraStatus1.Monitor", "Update", ())
    }
}

pub trait DeNlihDiveraStatus1Status {
    fn next(&self) -> Result<(), dbus::Error>;
    fn previous(&self) -> Result<(), dbus::Error>;
}

impl<'a, T: blocking::BlockingSender, C: std::ops::Deref<Target = T>> DeNlihDiveraStatus1Status
    for blocking::Proxy<'a, C>
{
    fn next(&self) -> Result<(), dbus::Error> {
        self.method_call("de.nlih.DiveraStatus1.Status", "Next", ())
    }

    fn previous(&self) -> Result<(), dbus::Error> {
        self.method_call("de.nlih.DiveraStatus1.Status", "Previous", ())
    }
}
