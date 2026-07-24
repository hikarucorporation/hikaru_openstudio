//! Puente genérico entre el `Receiver<MainThreadMsg>` que produce un host de
//! plugins (CLAP, VST3, ...) en un hilo bloqueante, y el `Subscription`/`Task`
//! de Iced que lo consume en el hilo de la UI.
//!
//! Factoriza el patrón que antes vivía duplicado dentro de `ClapHost::plugin_add`:
//! reenviar mensajes de un `std::sync::mpsc::Receiver` bloqueante hacia un stream
//! async de `smol::channel`, para que Iced pueda drenarlo vía `Task::run`.

use std::sync::mpsc::Receiver;

use iced::Task;
use smol::unblock;

/// Puentea un canal bloqueante (`Receiver<Msg>`, típicamente el lado de recepción
/// de un `Sender<Msg>` que vive en el hilo de audio o de scanning de un host)
/// hacia un `Task<Out>` de Iced.
///
/// - `Msg`: el tipo de mensaje interno del host (p. ej. `clap_host::MainThreadMessage`
///   o el análogo `vst3_host::MainThreadMessage`).
/// - `Out`: el tipo de mensaje externo que consume la UI (p. ej. `daw::Message`).
/// - `wrap`: convierte cada `Msg` recibido en un `Out`. Normalmente algo como
///   `move |msg| Message::MainThread(id, Box::new(msg))`.
///
/// Devuelve un `Task` combinado que:
/// 1. drena el receiver bloqueante en un hilo aparte (`unblock`), reenviando
///    cada mensaje a un canal async (`smol::channel::unbounded`), y
/// 2. corre ese stream async dentro del runtime de Iced, mapeando cada mensaje
///    con `wrap`.
///
/// Si el lado de Iced deja de escuchar (el stream se cierra), el hilo bloqueante
/// corta el loop en el primer `try_send` fallido, sin quedar colgado.
pub fn bridge_main_thread_channel<Msg, Out, F>(receiver: Receiver<Msg>, wrap: F) -> Task<Out>
where
    Msg: Send + 'static,
    Out: Send + 'static,
    F: Fn(Msg) -> Out + Send + 'static,
{
    let (sender, stream) = smol::channel::unbounded();

    Task::batch([
        Task::future(unblock(move || {
            for msg in receiver {
                if sender.try_send(msg).is_err() {
                    return;
                }
            }
        }))
        .discard(),
        Task::run(stream, wrap),
    ])
}
