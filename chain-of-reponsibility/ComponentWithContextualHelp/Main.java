/**
 * Main - Client Driver for Chain of Responsibility Pattern Demonstration.
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI contextual help system.
 * <a href="https://refactoring.guru/design-patterns/chain-of-responsibility">refactoring.guru</a>
 *
 * <h2>Scenario</h2>
 * A UI application shows contextual help when the user presses F1 (or hovers).
 * The help request starts at the focused component and travels <em>up</em> the UI
 * containment hierarchy until a component that has relevant help text is found.
 *
 * <pre>
 * UI Hierarchy (chain order, from first to last handler):
 *
 *   Window (global help — always handles if nobody else does)
 *   └── Panel (optional modal help)
 *       └── Dialog (optional wiki page)
 *           ├── okButton     (optional tooltip)
 *           └── cancelButton (optional tooltip)
 * </pre>
 *
 * <h2>Three scenarios demonstrated</h2>
 * <ol>
 *   <li><b>Button WITH tooltip</b>  — chain terminates at the Button (first link).</li>
 *   <li><b>Button WITHOUT tooltip</b> — chain passes through Dialog (which has a wiki
 *       page) and terminates there.</li>
 *   <li><b>No specific help anywhere</b> — chain travels all the way to the Window's
 *       global help guide.</li>
 * </ol>
 *
 * <h2>5-Pattern Comparison (Interview Cheat Sheet)</h2>
 * <pre>
 * Pattern                 Structure                  Flow
 * ─────────────────────── ────────────────────────── ─────────────────────────────────────
 * Composite               Tree (1 parent→many kids)  Top-down: ALL children execute.
 * Decorator               Nested onion (1→1→1)        All layers run, adding behaviour.
 * Chain of Responsibility Linked list (1→1→1)         Sequential + conditional: STOPS early.
 * Observer                1-to-Many (Publisher→many)  Broadcast: ALL subscribers notified.
 * Proxy                   1-to-1 stand-in             Gatekeeper: decides if/when to forward.
 * </pre>
 *
 * <h2>Key confusions cleared</h2>
 * <ul>
 *   <li><b>CoR vs. Observer</b>: Observer = broadcast (everyone gets it at once).
 *       CoR = relay race (one baton, one runner at a time, stops when done).</li>
 *   <li><b>CoR vs. Decorator</b>: Decorator always runs all layers.
 *       CoR stops at the first handler that can answer.</li>
 *   <li><b>CoR vs. Composite</b>: Composite propagates DOWN to all children.
 *       CoR propagates UP through parents, stopping early.</li>
 *   <li><b>CoR vs. Proxy</b>: Proxy wraps one specific object to control access.
 *       CoR is a dynamic chain where any link can terminate or pass on.</li>
 * </ul>
 */
public class Main {

    public static void main(String[] args) {

        // =====================================================================
        // Build the UI hierarchy (chain wiring)
        // =====================================================================
        //
        // The Window is the root — it has no parent (end of chain).
        // Each add() call sets the parent as the next handler in the chain.
        //
        //   okButton.container       = loginDialog
        //   cancelButton.container   = loginDialog
        //   loginDialog.container    = mainPanel
        //   mainPanel.container      = appWindow
        //
        Window appWindow = new Window("Welcome to the Help Center! Use the navigation bar to find topics.");

        // Panel with no modal help (will forward to Window)
        Panel mainPanel = new Panel();
        appWindow.add(mainPanel);

        // Dialog that knows its own wiki page (will terminate the chain if reached)
        Dialog loginDialog = new Dialog("https://example.com/docs/login-dialog");
        mainPanel.add(loginDialog);

        // A Button WITH a tooltip — will handle the request itself (chain stops at Button)
        Button okButton = new Button("Click OK to submit your login credentials.");
        loginDialog.add(okButton);

        // A Button WITHOUT a tooltip — will pass the request up to its parent Dialog
        Button cancelButton = new Button();
        loginDialog.add(cancelButton);

        // =====================================================================
        // Scenario 1: Help requested on the OK button (HAS tooltip)
        // Expected: chain terminates immediately at Button (1st link)
        // =====================================================================
        printScenario(1, "F1 pressed on OK Button (has tooltip)",
                "Chain stops at Button — most specific answer wins.");
        okButton.showHelp();

        // =====================================================================
        // Scenario 2: Help requested on the Cancel button (NO tooltip)
        // Expected: Button passes → Dialog has a wiki page → chain stops at Dialog
        // =====================================================================
        printScenario(2, "F1 pressed on Cancel Button (no tooltip)",
                "Chain passes Button → stops at Dialog (wiki page found).");
        cancelButton.showHelp();

        // =====================================================================
        // Scenario 3: Panel with no help, Dialog with no help → reaches Window
        // Set up a fresh sub-hierarchy where nobody in between can handle it.
        // =====================================================================
        printScenario(3, "F1 pressed on a Button with no tooltip, inside a no-wiki Dialog, inside a no-modal Panel",
                "Chain travels all the way to the Window (global fallback).");

        Panel barePanel = new Panel();          // no modal help
        appWindow.add(barePanel);

        Dialog bareDialog = new Dialog();       // no wiki page
        barePanel.add(bareDialog);

        Button bareButton = new Button();       // no tooltip
        bareDialog.add(bareButton);

        bareButton.showHelp();

        // =====================================================================
        // Summary
        // =====================================================================
        System.out.println();
        System.out.println("=================================================================");
        System.out.println("   Chain of Responsibility Demonstration Complete");
        System.out.println("=================================================================");
        System.out.println("Key takeaway: the chain is a relay race — it stops the moment");
        System.out.println("a handler can answer.  Compare to Observer (everyone notified)");
        System.out.println("and Proxy (one dedicated gatekeeper, not a chain).");
        System.out.println("=================================================================");
    }

    // -------------------------------------------------------------------------
    // Helper
    // -------------------------------------------------------------------------

    /**
     * Prints a formatted scenario header for readability in the console output.
     *
     * @param number      scenario number.
     * @param title       brief description of the scenario.
     * @param expectation what we expect to happen.
     */
    private static void printScenario(int number, String title, String expectation) {
        System.out.println();
        System.out.println("=================================================================");
        System.out.printf("  Scenario %d: %s%n", number, title);
        System.out.printf("  Expected  : %s%n", expectation);
        System.out.println("=================================================================");
    }
}
