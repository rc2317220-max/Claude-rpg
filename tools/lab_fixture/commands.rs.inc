// Injected only into the disposable test candidate. Never in project/src/lib.rs.
"lab" => {
    if let CommandSource::Player{player,..}=&e.source {
        let uuid=player.uuid();
        let dim=player.entity().level(host).map(|v|v.dimension_id().to_owned())
            .unwrap_or_else(||host.dimension().to_owned());
        let entity=player.entity().id();
        let mut p=PROFILE.get(host,Holder::Player(uuid)).unwrap_or_else(|_|Profile::new());
        let op=e.command.literals.get(2).map(String::as_str).unwrap_or("snapshot");
        let mut result=String::from("snapshot");
        match op {
            "seed" => {
                // Fixture only: create deterministic resources on the throwaway world.
                p=Profile::new(); p.level=20; p.points=100; p.stamina.current=p.stamina.max;
                rustpg_guard::remove(&dim,entity); rustpg_powers::remove(&dim,entity);
                result="seed".into();
            }
            "refill" => {p.stamina.current=p.stamina.max;p.stamina.exhausted=false;result="refill".into();}
            "buy_fury"|"buy_barrier"|"buy_siphon" => {
                let id=match op {"buy_fury"=>"power.fury","buy_barrier"=>"power.barrier",_=>"power.siphon"};
                result=match p.buy(id){Ok(n)=>format!("bought:{id}:{n}"),Err(_)=>format!("refused:{id}")};
            }
            _=>{}
        }
        if op!="snapshot" {
            sync_player_stats(uuid,&p.node_stats());
            if PROFILE.set(host,Holder::Player(uuid),p.clone()).is_err(){result="save_failed".into();}
        }
        let stats=p.node_stats();
        let _=host.commands().feedback(Text::literal(format!(
            "RPG_LAB tick={} op={} points={} stamina={:.4} max={:.4} fury={} barrier={} siphon={} fury_stat={:.4} outgoing={:.4} incoming={:.4} siphon10={:.4} guard={} creative={}",
            rustpg_guard::current_tick(),result,p.points,p.stamina.current,p.stamina.max,
            p.rank("power.fury"),p.rank("power.barrier"),p.rank("power.siphon"),stats.get("power.fury"),
            rustpg_powers::outgoing(&dim,entity),rustpg_powers::incoming(&dim,entity),
            rustpg_powers::siphon(&dim,entity,10.0),rustpg_guard::lab_state(&dim,entity),
            player.is_creative(host).unwrap_or(false))));
    }
},
