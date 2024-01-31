// * ‘server’: 开启TcpListner监听，处理过来的请求
// * ‘client’ 向server发起请求，set，get等命令，可以拿到结果
// * 'command' 抽象出redis操作的各种命令
// * 'frame' 一个完整的redis请求抽象出来的结构体，类似于http中的header，body等

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
