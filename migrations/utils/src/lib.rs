pub trait MigrateInto<T> {
    fn migrate(self) -> T;
}

impl<T> MigrateInto<T> for T {
    fn migrate(self) -> T {
        self
    }
}
