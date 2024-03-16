#![allow(non_snake_case)]

use std::future::Future;
use tokio::runtime::{Runtime, Builder};
use std::sync::Arc;
use std::thread;
use lazy_static::lazy_static;

lazy_static! {
    static ref GLOBAL_RUNTIME: Arc<Runtime> = Arc::new(
        Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    );
    static ref GLOBAL_RUNTIME_HANDLE: Arc<GRuntimeTask> = Arc::new(GRuntimeTask::start());
}

pub struct GRuntimeTask {
    rt_handle: tokio::runtime::Handle,
    rt_task_sender: tokio::sync::mpsc::UnboundedSender<(tokio::task::JoinHandle<()>, String)>,
}

impl GRuntimeTask {
    pub fn start() -> Self {
        let rt_handle = GLOBAL_RUNTIME.handle().clone();

        // 创建一个MPSC通道用于收集任务
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(tokio::task::JoinHandle<()>, String)>();

        // 克隆运行时的句柄
        let rt_handle2 = rt_handle.clone();

        // 在另一个线程中监听任务收集器
        thread::spawn(move || {
            rt_handle.block_on(async {
                while let Some((task, name)) = rx.recv().await {
                    // if name == "blocking_task" {
                    //     // 当有新的任务时，调度执行
                    //     task.await.unwrap();
                    //     continue;
                    // }
                    // 当有新的任务时，调度执行
                    tokio::select! {
                    _ = task => {
                        println!("任务 {} 完成", name);
                    }
                }
                }
            });
        });

        Self {
            rt_handle:rt_handle2,
            rt_task_sender:tx,
        }
    }

    pub fn stop(self) {
        drop(self);
    }

    // fn spawn_task_with_return<T>(&self,future: JoinHandle<T>)
    //     where
    //         T: Send + 'static, // 确保T满足Send trait和'static生命周期
    // {
    //     let rt_handle = self.rt_handle.clone();
    //     let task=rt_handle.spawn(future);
    //     self.rt_task_sender.send((task,"".to_string())).unwrap();
    // }

    // 可以在任何地方调用这个方法，传入一个实现了Future trait的future
    pub fn task<F>(&self,future: F)
        where
            F: Future + Send + 'static,
            F: Future<Output = ()> + Send + 'static, // 约束 F::Output 为 ()
    {
        let rt_handle = self.rt_handle.clone();
        let task=rt_handle.spawn(future);
        self.rt_task_sender.send((task,"task".to_string())).unwrap();
    }

    pub fn blocking_task<F>(&self,future: F)
        where
            F: Future + Send + 'static,
            F: Future<Output = ()> + Send + 'static, // 约束 F::Output 为 ()
    {
        let rt_handle = self.rt_handle.clone();
        let task=rt_handle.spawn(future);
        self.rt_task_sender.send((task,"blocking_task".to_string())).unwrap();
    }

    // 心跳任务：可以指定多长时间执行一次。todo:无法使用
    // pub fn heartbeat_task<F>(&self, interval_secs: u64, future: F)
    //     where
    //         F: Future + Send + 'static,
    //         F: Future<Output=()> + Send + 'static, // 约束 F::Output 为 ()
    // {
    //     let rt_handle = self.rt_handle.clone();
    //     let task = rt_handle.spawn(async move {
    //         let mut interval = Duration::from_secs(interval_secs);
    //         loop {
    //             sleep(interval).await;//sleep的方式可能会漂移：如果任务执行所需时间较长，下一次任务的实际执行时间可能会延迟。
    //             future.await;
    //             interval = Duration::from_secs(interval_secs);
    //         }
    //     });
    //     self.rt_task_sender.send((task, "heartbeat_task".to_string())).unwrap();
    // }

    // tokio::time::sleep(delay).await;可以放在future中，也可以放在这里
    pub fn call_later<F>(&self, future: F, delay: std::time::Duration)
        where
            F: Future + Send + 'static,
            F: Future<Output=()> + Send + 'static, // 约束 F::Output 为 ()
    {
        let rt_handle = self.rt_handle.clone();
        let task = rt_handle.spawn(async move {
            tokio::time::sleep(delay).await;
            future.await;
        });
        self.rt_task_sender.send((task, "call_later".to_string())).unwrap();
    }
}

pub fn GRuntime() -> Arc<GRuntimeTask> {
    GLOBAL_RUNTIME_HANDLE.clone()
}