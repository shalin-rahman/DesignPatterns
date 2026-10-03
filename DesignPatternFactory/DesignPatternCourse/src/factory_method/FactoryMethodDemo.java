package factory_method;

public class FactoryMethodDemo {
    public static void main(String[] args) {
        NotificationService[] services = {
            new EmailService(),
            new SmsService(),
            new PushService()
        };
        String[] to = { "ana@shop.com", "555-0101", "device-42" };
        String msg = "  Order shipped ";

        for (int i = 0; i < services.length; i++) {
            services[i].notifyUser(to[i], msg);
        }
    }
}
