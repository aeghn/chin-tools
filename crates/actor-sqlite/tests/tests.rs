use actor_sqlite::pool_config::PoolConfig;

#[tokio::test]
async fn test1() {
    let pool = actor_sqlite::pool::ActorSqlitePool::try_from(PoolConfig::default()).unwrap();
    for _ in 1..10 {
        let client = pool.get().await.unwrap();
        tokio::spawn(async move {
            let r = client
                .query("create table emacs(id number) error", [].to_vec())
                .await;
            println!("error: {r:?}")
        });
        let client = pool.get().await.unwrap();

        let r = client
            .query("create table emacs(id number) error", [].to_vec())
            .await;
        println!("error: {r:?}")
    }
}
