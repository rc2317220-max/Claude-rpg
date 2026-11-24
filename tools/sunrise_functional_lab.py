#!/usr/bin/env python3
"""Run real Sunrise control commands on a DISPOSABLE Android world. No mocks.
Keep raw before/after values. Tests which cannot prove an outcome are BLOCKED,
never passed just because a command returned OK."""
import argparse,json,os,socket,subprocess,time
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
def guard():
    before=cmd("status");ok("key b tap");ok("key g tap");time.sleep(.5);after=cmd("status");events=cmd("events --limit 30");shot("guard-input")
    # No stable public RustPG guard telemetry yet: refuse to claim actual parry/dodge.
    raise RuntimeError("B/G input recorded; need authoritative stamina/guard-window telemetry. See control.jsonl")
def skill():
    before=debug();ok("key v tap");time.sleep(1);u=cmd("ui");shot("skill-ui-inspection");ok("key esc tap")
    raise RuntimeError("skill UI captured; purchase and stat delta require an unlocked node and a stable widget/fixture, not a guessed click")
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
        check("guard_dodge_observation",guard)
        check("skill_purchase_observation",skill)
finally:
    log.close();s.close();subprocess.run(["adb","forward","--remove",f"tcp:{port}"],check=False)
if any(x["result"]!="PASS" for x in results):raise SystemExit(1)
