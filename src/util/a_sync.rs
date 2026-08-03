use tokio::time::{sleep, Duration};

pub fn drop_later<T: Send + 'static>(value: T, time: Duration) {
    tokio::spawn(async move {
        sleep(time).await;
        drop(value);
    });
}
