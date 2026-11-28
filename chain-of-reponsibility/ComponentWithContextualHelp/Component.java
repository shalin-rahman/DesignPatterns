/**
 * Component - Abstract Handler (Base Class).
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 * <a href="https://refactoring.guru/design-patterns/chain-of-responsibility">refactoring.guru</a>
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>Declares the interface for all concrete handlers in the chain.</li>
 *   <li>Holds a reference to the <em>next</em> handler ({@code container}), forming a
 *       singly-linked list (the "chain").</li>
 *   <li>Provides the default "pass-along" behaviour: if this component cannot handle the
 *       request, it forwards it to its parent container.</li>
 * </ul>
 *
 * <h2>Pattern vs. Observer &amp; Proxy (Why This Is Different)</h2>
 * <ul>
 *   <li><b>Observer (1-to-Many broadcast)</b>: a Publisher notifies <em>all</em> subscribers
 *       at once — no one passes anything on.  Here, the help request travels <em>one hop at
 *       a time</em> and stops the moment someone handles it.</li>
 *   <li><b>Proxy (1-to-1 stand-in)</b>: a Proxy <em>always</em> delegates to exactly one
 *       Real Subject.  Here, each handler independently decides whether to handle or forward.</li>
 *   <li><b>Chain of Responsibility (1-to-1-to-1 relay)</b>: sequential, conditional
 *       forwarding — only one link in the chain ultimately handles the request.</li>
 * </ul>
 *
 * <h2>Analogy (Restaurant)</h2>
 * Complaining to the Waiter → escalates to Head Chef → escalates to Manager.
 * Each link decides: "Can I fix this?" If yes, it stops; if no, it passes on.
 */
public abstract class Component {

    /**
     * Optional tooltip / help text this component can display.
     */
    private String tooltipText;

    /**
     * The next handler in the chain — typically the parent UI container
     * (e.g., a Button's parent is a Dialog; a Dialog's parent is a Panel).
     *
     * <p>This reference is what transforms a flat set of objects into a linked chain.
     */
    protected Component container;

    // -------------------------------------------------------------------------
    // Chain construction
    // -------------------------------------------------------------------------

    /**
     * Wires this component into the chain by setting its parent container as the
     * next handler.  Called once during UI hierarchy construction.
     *
     * @param container the enclosing UI container that acts as the next handler.
     */
    public void setContainer(Component container) {
        this.container = container;
    }

    // -------------------------------------------------------------------------
    // Handler logic
    // -------------------------------------------------------------------------

    /**
     * Attempts to show contextual help for this component.
     *
     * <p><b>Default behaviour (pass-along)</b>: if this component has tooltip text, it
     * displays it and the chain stops here.  Otherwise, the request is forwarded up to
     * {@link #container} — exactly one hop per handler, one handler per turn.
     *
     * <p>This is the core CoR mechanism: the handler either <em>handles</em> or
     * <em>passes</em>, never both simultaneously.
     */
    public void showHelp() {
        if (tooltipText != null) {
            // This handler CAN answer the request — chain stops here.
            System.out.println("[" + getClass().getSimpleName() + "] Showing tooltip: \"" + tooltipText + "\"");
        } else if (container != null) {
            // This handler CANNOT answer — pass the request to the next link.
            System.out.println("[" + getClass().getSimpleName() + "] No tooltip — forwarding to " + container.getClass().getSimpleName() + "...");
            container.showHelp();
        } else {
            // End of chain — nobody handled the request.
            System.out.println("[" + getClass().getSimpleName() + "] End of chain — no help available.");
        }
    }

    // -------------------------------------------------------------------------
    // Tooltip configuration
    // -------------------------------------------------------------------------

    /**
     * Assigns a tooltip (help text) to this component, making it a terminating handler.
     *
     * @param text the help text to display when this component is the first in the chain
     *             that can respond.
     */
    public void setTooltipText(String text) {
        this.tooltipText = text;
    }

    /**
     * Returns the tooltip text, or {@code null} if none is set.
     *
     * @return tooltip string, may be null.
     */
    public String getTooltipText() {
        return tooltipText;
    }
}
