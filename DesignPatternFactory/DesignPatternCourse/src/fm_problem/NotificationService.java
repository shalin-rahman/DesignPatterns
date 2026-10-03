package fm_problem;

public class NotificationService {

    public NotificationService() {
    }

    public void notifyUser(String channel, String to, String message) {
        // creation code ...
        Notification n;
        if (channel.equals("email")) {
            n = new EmailNotification();
        } else if (channel.equals("sms")) {
            n = new SmsNotification();
        } else {
            throw new IllegalArgumentException(
                    "Unknown channel: " + channel);
        }

        // ... mixed with business logic
        String text = message.trim();
        System.out.println("[log] via " + channel);
        n.send(to, text);
    }
}
