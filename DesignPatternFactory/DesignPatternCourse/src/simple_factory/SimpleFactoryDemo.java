package simple_factory;

public class SimpleFactoryDemo {
    public static void main(String[] args) {
        NotificationService service = new NotificationService();
        String msg = "Order shipped";

        service.notifyUser("email", "ana@shop.com", msg);
        service.notifyUser("sms", "555-0101", msg);
        service.notifyUser("push", "device-42", msg);
    }
}
