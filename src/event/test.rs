use crate::{
    coord::Coord2,
    event::{channel, Event, Key, KeyEvent, Listener, Reactor, ResizeEvent},
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
