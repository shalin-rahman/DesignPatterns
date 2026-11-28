/**
 * Client driver assembling the UI hierarchy and demonstrating Chain of Responsibility scenarios.
 */
public class Main {

    public static void main(String[] args) {
        // Wire the UI hierarchy: Button -> Dialog -> Panel -> Window
        Window appWindow = new Window("Welcome to the Help Center! Use the navigation bar to find topics.");

        Panel mainPanel = new Panel(); // no modal help (forwards to Window)
        appWindow.add(mainPanel);

        Dialog loginDialog = new Dialog("https://example.com/docs/login-dialog"); // has wiki URL
        mainPanel.add(loginDialog);

        Button okButton = new Button("Click OK to submit your login credentials."); // has tooltip
        loginDialog.add(okButton);

        Button cancelButton = new Button(); // no tooltip
        loginDialog.add(cancelButton);

        // Scenario 1: Button with tooltip handles request immediately (link 1 early stop)
        printScenario(1, "F1 pressed on OK Button (has tooltip)",
                "Chain stops at Button — most specific answer wins.");
        okButton.showHelp();

        // Scenario 2: Button without tooltip forwards to Dialog (link 2 stop)
        printScenario(2, "F1 pressed on Cancel Button (no tooltip)",
                "Chain passes Button → stops at Dialog (wiki page found).");
        cancelButton.showHelp();

        // Scenario 3: Neither button, dialog, nor panel has help -> reaches root Window fallback
        printScenario(3, "F1 pressed on a Button with no tooltip, inside a no-wiki Dialog, inside a no-modal Panel",
                "Chain travels all the way to the Window (global fallback).");

        Panel barePanel = new Panel();
        appWindow.add(barePanel);

        Dialog bareDialog = new Dialog();
        barePanel.add(bareDialog);

        Button bareButton = new Button();
        bareDialog.add(bareButton);

        bareButton.showHelp();

        System.out.println();
        System.out.println("=================================================================");
        System.out.println("   Chain of Responsibility Demonstration Complete");
        System.out.println("=================================================================");
    }

    private static void printScenario(int number, String title, String expectation) {
        System.out.println();
        System.out.println("=================================================================");
        System.out.printf("  Scenario %d: %s%n", number, title);
        System.out.printf("  Expected  : %s%n", expectation);
        System.out.println("=================================================================");
    }
}
