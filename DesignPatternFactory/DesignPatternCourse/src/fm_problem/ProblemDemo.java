package fm_problem;

public class ProblemDemo {
    public static void main(String[] args) {
        NotificationService service = new NotificationService();
        
        String msg = "Order shipped";

        service.notifyUser("email", "ana@shop.com", msg);
        service.notifyUser("sms", "555-0101", msg);

        try {  // marketing asks for push notifications...
            service.notifyUser("push", "device-42", msg);
        } catch (IllegalArgumentException e) {
            System.out.println("ERROR: " + e.getMessage());
        }
    }
}
