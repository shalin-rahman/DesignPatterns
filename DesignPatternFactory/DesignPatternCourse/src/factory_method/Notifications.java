package factory_method;

interface Notification {
    void send(String to, String msg);
}

class EmailNotification implements Notification {
    public void send(String to, String msg) {
        System.out.printf("EMAIL -> %s: %s%n", to, msg);
    }
}

class SmsNotification implements Notification {
    public void send(String to, String msg) {
        System.out.printf("SMS   -> %s: %s%n", to, msg);
    }
}

class PushNotification implements Notification {
    public void send(String to, String msg) {
        System.out.printf("PUSH  -> %s: %s%n", to, msg);
    }
}
