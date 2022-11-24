pub mod tcp;

pub fn demo_all()-> anyhow::Result<()> {
    tcp::test_tcp()?;

    tcp::test_tcp_bind()?;

    tcp::test_tcp_bind_async()?;
    Ok(())
}