/**
 * Main - Client Driver for Observer Pattern Demonstration.
 *
 * Demonstrates:
 *   1. Initial subscription setup (binding listeners to specific event types).
 *   2. Event dispatching through Editor actions ("open" and "save").
 *   3. Dynamic unsubscription showing observers can be detached at runtime.
 *   4. Exception safety when attempting operations without prerequisites.
 */
public class Main {

    public static void main(String[] args) {
        System.out.println("=================================================================");
        System.out.println("   Refactoring Guru: EventManager Observer Pattern Demonstration");
        System.out.println("=================================================================\n");

        Editor editor = new Editor();

        // 1. Create observers
        LogOpenListener logger = new LogOpenListener("app_events.log");
        EmailNotificationListener adminEmail = new EmailNotificationListener("admin@example.com");
        EmailNotificationListener devEmail = new EmailNotificationListener("devops@example.com");

        // 2. Register subscriptions
        System.out.println("[Step 1] Subscribing listeners to events...");
        editor.events.subscribe("open", logger);
        editor.events.subscribe("open", devEmail);
        editor.events.subscribe("save", adminEmail);
        editor.events.subscribe("save", devEmail);
        System.out.println(" -> 'open' event subscribed by: LogOpenListener, DevOps Email");
        System.out.println(" -> 'save' event subscribed by: Admin Email, DevOps Email\n");

        // 3. Trigger events via Editor
        System.out.println("[Step 2] Executing Editor operations...");
        try {
            System.out.println("\n--- Opening 'document.pdf' ---");
            editor.openFile("document.pdf");

            System.out.println("\n--- Saving 'document.pdf' ---");
            editor.saveFile();
        } catch (Exception e) {
            System.err.println("Error during file operation: " + e.getMessage());
        }

        // 4. Dynamic Unsubscription
        System.out.println("\n[Step 3] Dynamic Unsubscription: Removing DevOps from 'open' and 'save'...");
        editor.events.unsubscribe("open", devEmail);
        editor.events.unsubscribe("save", devEmail);

        try {
            System.out.println("\n--- Opening 'report.xlsx' ---");
            editor.openFile("report.xlsx");

            System.out.println("\n--- Saving 'report.xlsx' ---");
            editor.saveFile();
        } catch (Exception e) {
            System.err.println("Error during file operation: " + e.getMessage());
        }

        // 5. Error handling demonstration
        System.out.println("\n[Step 4] Testing error handling: Saving with uninitialized editor...");
        Editor emptyEditor = new Editor();
        try {
            emptyEditor.saveFile();
        } catch (Exception e) {
            System.out.println("Expected error caught successfully: " + e.getMessage());
        }

        System.out.println("\n=================================================================");
        System.out.println("   Observer Pattern Demonstration Completed Successfully");
        System.out.println("=================================================================");
    }
}
