use crate::{
    coord::Coord2,
    event::{
        channel,
        Event,
        Key,
        KeyEvent,
        Listener,
        Reactor,
        ReactorSubs,
        ResizeEvent,
    },
};
use std::sync::Arc;
use tokio::{
    sync::{Barrier, Notify},
    task,
};

#[tokio::test]
async fn singletask() {
    let (reactor, mut listener) = channel();
    assert_eq!(listener.check().unwrap(), None);

    singletask_resize(&reactor, &mut listener).await;
    singletask_key(&reactor, &mut listener).await;
    singletask_shadow_keys(&reactor, &mut listener).await;
    singletask_shadow_mixed(&reactor, &mut listener).await;
    singletask_mixed(&reactor, &mut listener).await;
}

#[tokio::test]
async fn multitask() {
    const LISTENERS: usize = 3;

    let notify = Arc::new(Notify::new());
    let barrier = Arc::new(Barrier::new(LISTENERS + 1));
    let mut listener_threads = Vec::new();
    let (reactor, listener) = channel();

    for _ in 0 .. LISTENERS {
        let handle = task::spawn({
            let notify = notify.clone();
            let barrier = barrier.clone();
            let listener = listener.clone();
            multitask_listener(listener, notify, barrier)
        });
        listener_threads.push(handle);
    }

    drop(listener);

    let reactor_thread =
        task::spawn(multitask_reactor(reactor, LISTENERS, notify, barrier));

    let (res0, res1, res2, res_r) = tokio::join!(
        listener_threads.pop().unwrap(),
        listener_threads.pop().unwrap(),
        listener_threads.pop().unwrap(),
        reactor_thread
    );
    assert_eq!(listener_threads.len(), 0);
    res0.unwrap();
    res1.unwrap();
    res2.unwrap();
    res_r.unwrap();
}

async fn multitask_listener(
    mut listener: Listener,
    notify: Arc<Notify>,
    barrier: Arc<Barrier>,
) {
    multitask_recv_resizes(&mut listener, &notify).await;
    barrier.wait().await;
    multitask_recv_keys(&mut listener, &notify).await;
    barrier.wait().await;
    multitask_recv_mixed(&mut listener, &notify).await;
}

async fn multitask_reactor(
    reactor: Reactor,
    listeners: usize,
    notify: Arc<Notify>,
    barrier: Arc<Barrier>,
) {
    multitask_send_resizes(&reactor).await;
    for _ in 0 .. listeners {
        notify.notify_one();
    }
    barrier.wait().await;

    multitask_send_keys(&reactor).await;
    for _ in 0 .. listeners {
        notify.notify_one();
    }
    barrier.wait().await;

    multitask_send_mixed(&reactor).await;
    for _ in 0 .. listeners {
        notify.notify_one();
    }
    ReactorSubs { reactor: &reactor }.await;
}

async fn multitask_recv_resizes(listener: &mut Listener, notify: &Notify) {
    loop {
        let message = tokio::select! {
            msg = listener.listen() => msg,
            _ = notify.notified() => break,
        };

        match message.unwrap() {
            Event::Resize(ResizeEvent { size: Some(size) }) => {
                assert!(size.x < 200);
                assert!(size.y < 100);
            },
            evt => panic!("{:#?}", evt),
        }
    }

    listener.check().unwrap();
}

async fn multitask_recv_keys(listener: &mut Listener, notify: &Notify) {
    loop {
        let message = tokio::select! {
            msg = listener.listen() => msg,
            _ = notify.notified() => break,
        };

        match message.unwrap() {
            Event::Key(KeyEvent {
                main_key: Key::Char(ch),
                shift: true,
                ..
            }) => {
                assert!(ch >= 'a' && ch <= 'z' || ch >= 'A' && ch <= 'Z');
            },
            evt => panic!("{:#?}", evt),
        }
    }

    listener.check().unwrap();
}

async fn multitask_recv_mixed(listener: &mut Listener, notify: &Notify) {
    loop {
        let message = tokio::select! {
            msg = listener.listen() => msg,
            _ = notify.notified() => break,
        };

        match message.unwrap() {
            Event::Key(KeyEvent {
                main_key: Key::Char(ch),
                alt: false,
                ctrl: true,
                shift: false,
                ..
            }) => {
                assert!(ch >= 'a' && ch <= 'z' || ch >= 'A' && ch <= 'Z');
            },
            Event::Resize(ResizeEvent { size: Some(Coord2 { x, y: 25 }) }) => {
                assert!(x < 500)
            },
            evt => panic!("{:#?}", evt),
        }
    }

    listener.check().unwrap();
}

async fn multitask_send_resizes(reactor: &Reactor) {
    for x in 0 .. 200 {
        for y in 0 .. 100 {
            let size = Some(Coord2 { x, y });
            reactor.send(Event::Resize(ResizeEvent { size }));
        }
        task::yield_now().await;
    }
}

async fn multitask_send_keys(reactor: &Reactor) {
    for ch in ('A' ..= 'Z').chain('a' ..= 'z') {
        for &alt in &[false, true] {
            for &ctrl in &[false, true] {
                let key = KeyEvent {
                    main_key: Key::Char(ch),
                    alt,
                    ctrl,
                    shift: true,
                };
                reactor.send(Event::Key(key));
            }
        }
        task::yield_now().await;
    }
}

async fn multitask_send_mixed(reactor: &Reactor) {
    let y = 25;
    let chars = ('A' ..= 'Z').chain('a' ..= 'z').cycle();
    for (x, ch) in (0 .. 500).zip(chars) {
        let size = Some(Coord2 { x, y });
        reactor.send(Event::Resize(ResizeEvent { size }));
        let key = KeyEvent {
            main_key: Key::Char(ch),
            alt: false,
            ctrl: true,
            shift: false,
        };
        reactor.send(Event::Key(key));
        if (x + 1) % 100 == 0 {
            task::yield_now().await;
        }
    }
}

async fn singletask_resize(reactor: &Reactor, listener: &mut Listener) {
    let event = Event::Resize(ResizeEvent { size: None });
    reactor.send(event);
    assert_eq!(listener.check().unwrap(), Some(event));
    assert_eq!(listener.check().unwrap(), None);

    let size = Some(Coord2 { x: 80, y: 80 });
    let event = Event::Resize(ResizeEvent { size });
    reactor.send(event);
    assert_eq!(listener.listen().await.unwrap(), event);
    assert_eq!(listener.check().unwrap(), None);
}

async fn singletask_key(reactor: &Reactor, listener: &mut Listener) {
    let event = Event::Key(KeyEvent {
        main_key: Key::Char(' '),
        alt: false,
        ctrl: true,
        shift: false,
    });

    reactor.send(event);
    assert_eq!(listener.listen().await.unwrap(), event);
    assert_eq!(listener.check().unwrap(), None);

    let event = Event::Key(KeyEvent {
        main_key: Key::Char('d'),
        alt: true,
        ctrl: false,
        shift: false,
    });

    reactor.send(event);
    assert_eq!(listener.check().unwrap(), Some(event));
    assert_eq!(listener.check().unwrap(), None);
}

async fn singletask_shadow_keys(reactor: &Reactor, listener: &mut Listener) {
    let shadowed = Event::Key(KeyEvent {
        main_key: Key::Char('F'),
        alt: false,
        ctrl: false,
        shift: true,
    });
    reactor.send(shadowed);

    let event = Event::Key(KeyEvent {
        main_key: Key::Char('u'),
        alt: false,
        ctrl: true,
        shift: false,
    });
    reactor.send(event);

    assert_eq!(listener.check().unwrap(), Some(event));
    assert_eq!(listener.check().unwrap(), None);

    let shadowed = Event::Key(KeyEvent {
        main_key: Key::Char('J'),
        alt: false,
        ctrl: false,
        shift: true,
    });
    reactor.send(shadowed);

    let event = Event::Key(KeyEvent {
        main_key: Key::Char('c'),
        alt: false,
        ctrl: true,
        shift: false,
    });
    reactor.send(event);

    assert_eq!(listener.listen().await.unwrap(), event);
    assert_eq!(listener.check().unwrap(), None);
}

async fn singletask_shadow_mixed(reactor: &Reactor, listener: &mut Listener) {
    let event = Event::Resize(ResizeEvent { size: None });
    reactor.send(event);

    let shadowed = Event::Key(KeyEvent {
        main_key: Key::Char('J'),
        alt: false,
        ctrl: false,
        shift: true,
    });
    reactor.send(shadowed);

    assert_eq!(listener.check().unwrap(), Some(event));
    assert_eq!(listener.check().unwrap(), None);

    let size = Some(Coord2 { x: 25, y: 80 });
    let event = Event::Resize(ResizeEvent { size });
    reactor.send(event);

    let shadowed = Event::Key(KeyEvent {
        main_key: Key::Char('H'),
        alt: false,
        ctrl: true,
        shift: false,
    });
    reactor.send(shadowed);

    assert_eq!(listener.listen().await.unwrap(), event);
    assert_eq!(listener.check().unwrap(), None);
}

async fn singletask_mixed(reactor: &Reactor, listener: &mut Listener) {
    let event = Event::Resize(ResizeEvent { size: None });
    reactor.send(event);

    assert_eq!(listener.check().unwrap(), Some(event));
    assert_eq!(listener.check().unwrap(), None);

    let no_shadowing = Event::Key(KeyEvent {
        main_key: Key::Char('J'),
        alt: false,
        ctrl: false,
        shift: true,
    });
    reactor.send(no_shadowing);

    assert_eq!(listener.listen().await.unwrap(), no_shadowing);
    assert_eq!(listener.check().unwrap(), None);
}