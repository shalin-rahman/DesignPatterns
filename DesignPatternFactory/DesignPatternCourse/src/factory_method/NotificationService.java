package factory_method;

public abstract class NotificationService {

    // the factory method: subclasses decide the product
    protected abstract Notification createNotification();

    public void notifyUser(String to, String message) {
        Notification n = createNotification();
        String text = message.trim();  // business logic
        System.out.println("[log] "
                + getClass().getSimpleName());
        n.send(to, text);
    }
}
