/**
 * Button - Concrete Handler (Leaf node of the UI hierarchy).
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>A simple, non-container UI widget that can be the first link in the help chain.</li>
 *   <li>If the button has tooltip text assigned, {@link #showHelp()} (inherited from
 *       {@link Component}) handles the request here and the chain terminates.</li>
 *   <li>If no tooltip is set (e.g., a generic "OK" button), the request propagates to its
 *       parent container — the {@link Dialog} it lives inside.</li>
 * </ul>
 *
 * <h2>Chain position</h2>
 * <pre>
 *   Button → Dialog → Panel → Window
 *   (first link; request travels right until someone handles it)
 * </pre>
 *
 * <h2>Pattern Contrast</h2>
 * <ul>
 *   <li>In an <b>Observer</b> scenario, the Button would be a <em>Publisher</em>:
 *       pressing it would broadcast a "click" event to <em>all</em> registered listeners
 *       simultaneously (no forwarding, no stopping early).</li>
 *   <li>Here, the Button is a <em>handler</em>: it either owns the answer or passes
 *       the question on — strictly one-at-a-time, never broadcast.</li>
 * </ul>
 */
public class Button extends Component {

    /**
     * Constructs a Button without a tooltip (request will propagate to parent).
     */
    public Button() {
        // No tooltip by default — the chain will pass the request upward.
    }

    /**
     * Constructs a Button with pre-assigned tooltip text, making it a terminating handler.
     *
     * @param tooltipText the help text this button can display.
     */
    public Button(String tooltipText) {
        setTooltipText(tooltipText);
    }

    // showHelp() is fully inherited from Component.
    // The Button does NOT override it — the base "handle or forward" logic is sufficient.
}
