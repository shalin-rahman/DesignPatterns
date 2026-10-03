package simple_factory;

public class NotificationFactory {

    public static Notification create(String channel) {
        switch (channel) {
            case "email": return new EmailNotification();
            case "sms":   return new SmsNotification();
            case "push":  return new PushNotification(); // new
            default:
                throw new IllegalArgumentException(
                        "Unknown channel: " + channel);
        }
    }
}
