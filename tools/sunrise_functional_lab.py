#!/usr/bin/env python3
"""Run real Sunrise control commands on a DISPOSABLE Android world. No mocks.
Keep raw before/after values. Tests which cannot prove an outcome are BLOCKED,
never passed just because a command returned OK."""
import argparse,json,os,re,socket,subprocess,time
from pathlib import Path
ap=argparse.ArgumentParser();ap.add_argument("--endpoint",required=True);ap.add_argument("--device-home",required=True);ap.add_argument("--out",type=Path,required=True);ap.add_argument("--baseline",action="store_true");a=ap.parse_args()
a.out.mkdir(parents=True,exist_ok=True)
endpoint=json.loads(subprocess.check_output(["adb","shell","cat",a.endpoint],text=True))
port=int(subprocess.check_output(["adb","forward","tcp:0","tcp:"+str(endpoint["port"])],text=True).strip().removeprefix("tcp:"))
s=socket.create_connection(("127.0.0.1",port),30);s.settimeout(180)
r=s.makefile("rb");w=s.makefile("wb"); log=(a.out/"control.jsonl").open("w");results=[]
def cmd(line,secret=False):
    w.write((line+"\n").encode());w.flush()
    raw=r.readline()
    if not raw:raise RuntimeError("empty control reply")
    val=json.loads(raw)
    log.write(json.dumps({"command":"auth [redacted]" if secret else line,"reply":val},ensure_ascii=False)+"\n");log.flush()
    if not secret:print(line,str(val)[:500],flush=True)
    return val
def check(name,fn):
    try: evidence=fn();results.append(dict(test=name,result="PASS",evidence=evidence))
    except AssertionError as e:results.append(dict(test=name,result="FAIL",reason=str(e)))
    except Exception as e:results.append(dict(test=name,result="BLOCKED",reason=str(e)))
    (a.out/"results.json").write_text(json.dumps(results,indent=2,ensure_ascii=False))
def ok(line):
    v=cmd(line);assert v.get("ok") is True,(line,v);return v
def shot(name):
    path=a.device_home+"/"+name+".png"
    ok("screenshot "+path);subprocess.run(["adb","pull",path,str(a.out/(name+".png"))],check=True)
def msg():
    return cmd("messages 30")
def debug():
    ok("chat /rustpg debug");time.sleep(2);v=msg();assert "Perfil carregado" in json.dumps(v,ensure_ascii=False),v;return v
def menu():
    ok("key v tap");time.sleep(2);u=cmd("ui");shot("menu-v")
    assert u.get("screen")=="mod",u
    ok("key esc tap");time.sleep(1);assert cmd("ui").get("screen")!="mod"
    return u
def arena():
    ok("fill -14 179 -14 14 179 14 minecraft:stone");ok("fill -14 180 -14 14 195 14 minecraft:air");ok("tp 0 180 0")
def nearby(mid):
    return next((m for m in cmd("mobs 64").get("near",[]) if m.get("id")==mid),None)
def fight(kind):
    arena();v=ok("summon "+kind+" 0 180 2");mid=v["id"]
    time.sleep(1);before=nearby(mid);assert before and "health" in before,before
    x,y,z=before["pos"];ok(f"tp {x+1.5} {y+.1} {z}");ok(f"look-at entity {mid}");cmd("hurtlog")
    shot(kind.split(":")[-1]+"-before")
    ok(f"attack {mid}");time.sleep(2);after=nearby(mid);hurt=cmd("hurtlog");shot(kind.split(":")[-1]+"-after")
    assert after and float(after["health"])<float(before["health"]),(before,after,hurt)
    return {"before":before,"after":after,"damage":float(before["health"])-float(after["health"]),"hurtlog":hurt}
def effects():
    ok("effect clear");before=cmd("effects");ok("effect minecraft:speed 30 1");time.sleep(.5);after=cmd("effects")
    assert "speed" in json.dumps(after) and before!=after,(before,after)
    ok("effect clear");return {"before":before,"after":after,"clear":cmd("effects"),"scope":"vanilla control calibration, not RustPG ability"}
def snapshot(op="snapshot"):
    before=msg()
    ok("chat /rustpg lab "+op)
    for _ in range(20):
        time.sleep(.25)
        current=msg()
        if current==before:continue
        raw=json.dumps(current,ensure_ascii=False)
        rows=re.findall(r'RPG_LAB tick=\d+ op=[^ ]+ points=\d+ stamina=[\d.]+ max=[\d.]+ fury=\d+ barrier=\d+ siphon=\d+ fury_stat=[\d.]+ outgoing=[\d.]+ incoming=[\d.]+ siphon10=[\d.]+ guard=[^ ]+ creative=(?:true|false)',raw)
        if rows:
            # Most recent tick, and the intended op when relevant; do not reuse earlier evidence.
            candidates=[dict(v.split("=",1) for v in row.split()[1:]) for row in rows]
            matches=[v for v in candidates if (v["op"]==op or (op.startswith("buy_") and v["op"].startswith(("bought:","refused:")) and "power."+op[4:] in v["op"]))]
            if matches:return max(matches,key=lambda v:int(v["tick"]))
    raise RuntimeError("missing lab telemetry for "+op)
def wait_tick(target,timeout=180):
    deadline=time.monotonic()+timeout
    while time.monotonic()<deadline:
        v=snapshot()
        if int(v["tick"])>=target:return v
    raise RuntimeError("simulation did not advance to tick "+str(target))
def survival():
    ok("chat /gamemode survival")
    time.sleep(.5);s=snapshot()
    assert s["creative"]=="false",s
def guard():
    survival();seed=snapshot("seed");before=snapshot("refill")
    ok("key b tap");after=snapshot()
    assert after["guard"].startswith("aparar:"),after
    assert float(after["stamina"])<float(before["stamina"]),(before,after)
    ok("key b tap");cooldown=snapshot()
    assert after["guard"].split(":")[1:4]==cooldown["guard"].split(":")[1:4],(after,cooldown)
    wait_tick(int(after["guard"].split(":")[3])+1)
    before_dodge=snapshot("refill");ok("key g tap");dodge=snapshot()
    assert dodge["guard"].startswith("esquivar:"),dodge
    assert float(dodge["stamina"])<float(before_dodge["stamina"])
    shot("guard-input")
    return {"parry_before":before,"parry":after,"cooldown":cooldown,"dodge":dodge,"scope":"real key -> server payload -> actual window and cost; damage prevention separately covered by domain tests"}
def skill():
    survival();seed=snapshot("seed")
    assert seed["fury"]=="0" and seed["points"]=="100",seed
    ok("key j tap");locked=snapshot();assert float(locked["outgoing"])==1,locked
    bought=snapshot("buy_fury")
    assert bought["fury"]=="1" and int(seed["points"])-int(bought["points"])==8,(seed,bought)
    assert float(bought["fury_stat"])>float(seed["fury_stat"])
    denied=snapshot("buy_fury");assert denied["fury"]=="1" and denied["points"]==bought["points"],denied
    before=snapshot("refill");ok("key j tap");active=snapshot()
    assert abs(float(active["outgoing"])-1.25)<.001 and float(active["stamina"])<float(before["stamina"]),(before,active)
    shot("fury-active")
    expired=wait_tick(int(active["tick"])+81);assert float(expired["outgoing"])==1,expired
    barrier=snapshot("buy_barrier");assert barrier["barrier"]=="1" and int(denied["points"])-int(barrier["points"])==12,barrier
    before_barrier=snapshot("refill");ok("key k tap");shield=snapshot()
    assert abs(float(shield["incoming"])-.72)<.001 and float(shield["stamina"])<float(before_barrier["stamina"]),shield
    shot("barrier-active")
    siphon=snapshot("buy_siphon");assert siphon["siphon"]=="1" and int(barrier["points"])-int(siphon["points"])==16,siphon
    before_siphon=snapshot("refill");ok("key l tap");healing=snapshot()
    assert abs(float(healing["siphon10"])-.9)<.001 and float(healing["stamina"])<float(before_siphon["stamina"]),healing
    shot("siphon-active")
    return {"seed":seed,"locked":locked,"purchase":bought,"duplicate_denied":denied,"fury":active,"expired":expired,"barrier":shield,"siphon":healing,"scope":"server Profile::buy plus real J/K/L payload, multiplier/cost/expiry; not a UI purchase or final melee damage comparison"}
try:
    cmd("auth "+endpoint["token"],True);hello=cmd("hello");status=cmd("status")
    if not status.get("player"):ok("open RustPG_Preview4_Lab 424242 creative")
    # Raw TCP open is deferred until world loads; poll status instead of inventing a CLI command.
    for _ in range(90):
        if cmd("status").get("player"):break
        time.sleep(2)
    else:raise RuntimeError("world not loaded")
    shot("world");check("vanilla_effect_roundtrip",effects)
    if not a.baseline:
        check("rustpg_loaded_server_profile",debug)
        check("menu_v_and_escape",menu)
        check("sentinel_melee_damage",lambda:fight("rustpg:ashen_sentinel"))
        check("hound_melee_damage",lambda:fight("rustpg:mana_hound"))
        check("guard_dodge_window_cost_cooldown",guard)
        check("skill_purchase_powers_cost_effect_expiry",skill)
finally:
    log.close();s.close();subprocess.run(["adb","forward","--remove",f"tcp:{port}"],check=False)
if any(x["result"]!="PASS" for x in results):raise SystemExit(1)
