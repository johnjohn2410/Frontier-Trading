use anyhow::Result;
use redis::{aio::MultiplexedConnection, AsyncCommands};

pub async fn connect(url: &str) -> Result<MultiplexedConnection> {
    let client = redis::Client::open(url)?;
    Ok(client.get_multiplexed_async_connection().await?)
}

pub async fn xgroup_create_mkstream(
    conn: &mut MultiplexedConnection,
    key: &str,
    group: &str,
) -> Result<()> {
    let r: redis::RedisResult<redis::Value> = redis::cmd("XGROUP")
        .arg("CREATE").arg(key).arg(group).arg("$").arg("MKSTREAM")
        .query_async(conn).await;
    match r {
        Ok(_) => Ok(()),
        Err(e) if e.to_string().contains("BUSYGROUP") => Ok(()), // already exists
        Err(e) => Err(e.into()),
    }
}

pub async fn xreadgroup_block(
    conn: &mut MultiplexedConnection,
    group: &str,
    consumer: &str,
    streams: &[&str],
    count: usize,
    block_ms: u64,
) -> Result<redis::Value> {
    let mut cmd = redis::cmd("XREADGROUP");
    cmd.arg("GROUP").arg(group).arg(consumer)
        .arg("COUNT").arg(count)
        .arg("BLOCK").arg(block_ms)
        .arg("STREAMS");
    for s in streams { cmd.arg(*s); }
    for _ in streams { cmd.arg(">"); } // new messages only
    Ok(cmd.query_async(conn).await?)
}

pub async fn xack(
    conn: &mut MultiplexedConnection,
    key: &str,
    group: &str,
    id: &str,
) -> Result<()> {
    let _: i64 = redis::cmd("XACK").arg(key).arg(group).arg(id).query_async(conn).await?;
    Ok(())
}

pub async fn xclaim_justid(
    conn: &mut MultiplexedConnection,
    key: &str,
    group: &str,
    consumer: &str,
    min_idle_ms: u64,
    ids: &[&str],
) -> Result<Vec<String>> {
    let mut cmd = redis::cmd("XCLAIM");
    cmd.arg(key).arg(group).arg(consumer).arg(min_idle_ms);
    for id in ids { cmd.arg(*id); }
    cmd.arg("JUSTID");
    Ok(cmd.query_async(conn).await?)
}

pub async fn xpending_summary(
    conn: &mut MultiplexedConnection,
    key: &str,
    group: &str,
) -> Result<redis::Value> {
    Ok(redis::cmd("XPENDING").arg(key).arg(group).query_async(conn).await?)
}

pub async fn publish_tick(
    conn: &mut MultiplexedConnection,
    stream: &str,
    tick: &frontier_types::MarketTick,
) -> Result<String> {
    let _: String = conn.xadd_map(stream, "*", &[
        ("schema_version", &tick.schema_version),
        ("symbol", &tick.symbol),
        ("ts", &tick.ts.to_string()),
        ("price", &tick.price),
        ("source", &tick.source),
    ]).await?;
    Ok("published".to_string())
}
