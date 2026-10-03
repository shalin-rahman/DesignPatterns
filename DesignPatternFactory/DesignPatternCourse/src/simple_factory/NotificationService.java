package simple_factory;


public class NotificationService {

    public void notifyUser(String channel, String to,
                           String message) {
        Notification n = NotificationFactory.create(channel);
        String text = message.trim();
        System.out.println("[log] via " + channel);
        n.send(to, text);
    }
}
