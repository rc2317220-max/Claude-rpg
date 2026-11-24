//! Desktop-only JNI proof of concept; does not load arbitrary Minecraft mods.
use jni::{InitArgsBuilder, JNIEnv, JNIVersion, JavaVM, NativeMethod};
use jni::objects::{JClass, JObject, JValue};
use jni::sys::jint;
use std::{error::Error, ffi::c_void, path::Path};

extern "system" fn protocol_version(_: JNIEnv<'_>, _: JClass<'_>) -> jint { 1 }

fn checked_heal(hp: i32, max_hp: i32, requested: i32) -> i32 {
    if hp < 0 || hp > max_hp || !(0..=4).contains(&requested) { return hp; }
    hp.saturating_add(requested).min(max_hp)
}

fn run(api: &Path, jar: &Path) -> Result<(), Box<dyn Error>> {
    let classpath = format!("-Djava.class.path={}", api.canonicalize()?.display());
    let args = InitArgsBuilder::new().version(JNIVersion::V8).option(&classpath).build()?;
    let vm = JavaVM::new(args)?;
    let mut env = vm.attach_current_thread()?;
    let native = env.find_class("org/sunrise/bridge/NativeHost")?;
    env.register_native_methods(&native, &[NativeMethod {
        name: "protocolVersion".into(), sig: "()I".into(),
        fn_ptr: protocol_version as *mut c_void,
    }])?;
    let runtime = env.find_class("org/sunrise/bridge/BridgeRuntime")?;
    let jar = JObject::from(env.new_string(jar.canonicalize()?.to_string_lossy())?);
    let entry = JObject::from(env.new_string("example.HealingMod")?);
    let obj = env.call_static_method(&runtime, "open",
        "(Ljava/lang/String;Ljava/lang/String;)Lorg/sunrise/bridge/BridgeRuntime;",
        &[JValue::Object(&jar), JValue::Object(&entry)])?.l()?;
    let mut hp = 10;
    for tick in [1i64,20,40,60,80,100] {
        let request = env.call_method(&obj,"tick","(JII)I",
            &[JValue::Long(tick),JValue::Int(hp),JValue::Int(20)])?.i()?;
        hp = checked_heal(hp,20,request);
        println!("tick={tick} java_request={request} rust_health={hp}");
        let expected = match tick { 1 => 10, 20 => 12, 40|60|80 => 14, 100 => 16, _ => unreachable!() };
        if hp != expected { return Err(format!("wrong state at tick {tick}").into()); }
    }
    env.call_method(&obj,"close","()V",&[])?;
    println!("PASS: Rust -> JAR -> JNI callback -> Rust validation");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len()!=3 { return Err("usage: sunrise-java-bridge-lab <api.jar> <fixture.jar>".into()); }
    run(Path::new(&args[1]),Path::new(&args[2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn bounded_health() {
        assert_eq!(checked_heal(19,20,4),20);
        assert_eq!(checked_heal(10,20,-1),10);
        assert_eq!(checked_heal(10,20,999),10);
        assert_eq!(checked_heal(10,20,2),12);
    }
}
