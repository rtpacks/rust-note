// 餐厅前厅，用于吃饭

fn clean() {
    crate::cleanTable();
    super::cleanTable();
}

// 招待客人
pub mod hosting;

// 服务客人
mod serving;
