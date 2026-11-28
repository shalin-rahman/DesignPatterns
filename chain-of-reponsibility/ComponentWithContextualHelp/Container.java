import java.util.ArrayList;
import java.util.List;

/**
 * Container - Intermediate Abstract Handler (can hold child components).
 *
 * <p>Source: Refactoring Guru — Chain of Responsibility, GUI / Help-System example.
 *
 * <h2>Role in Chain of Responsibility Pattern</h2>
 * <ul>
 *   <li>Extends {@link Component}, inheriting the base "handle-or-forward" logic.</li>
 *   <li>Adds the ability to manage child components and automatically wires each child
 *       into the chain by calling {@link Component#setContainer(Component)} with
 *       {@code this} as the parent (next handler).</li>
 *   <li>Concrete containers ({@link Dialog}, {@link Panel}) extend this class and may
 *       override {@link #showHelp()} to provide container-level help before forwarding.</li>
 * </ul>
 *
 * <h2>Chain wiring mechanism</h2>
 * <pre>
 *   panel.add(dialog)  →  dialog.container = panel
 *   dialog.add(button) →  button.container = dialog
 *
 *   Request path: button → dialog → panel → window
 * </pre>
 *
 * <h2>Composite vs. Chain of Responsibility</h2>
 * <p>Both patterns use trees/hierarchies, which makes them easy to confuse:
 * <ul>
 *   <li><b>Composite</b>: Every node in the tree executes the operation
 *       ({@code draw()}, {@code getPrice()}).  All children participate.</li>
 *   <li><b>Chain of Responsibility</b>: The request travels <em>up</em> the hierarchy
 *       and stops the moment one handler claims it.  Children do <em>not</em> all participate.</li>
 * </ul>
 */
public abstract class Container extends Component {

    /**
     * Direct children of this container.
     * Adding a child auto-registers this container as the child's next handler.
     */
    private final List<Component> children = new ArrayList<>();

    /**
     * Adds a child component and sets this container as its parent (next handler in the chain).
     *
     * <p>This is the chain-wiring step: after {@code add()}, pressing F1 on the child
     * will propagate upward to this container if the child cannot handle it.
     *
     * @param child the UI component to nest inside this container.
     */
    public void add(Component child) {
        children.add(child);
        // Wire the chain: the child's "next handler" is this container.
        child.setContainer(this);
    }

    /**
     * Returns the list of direct children (read-only view, for demonstration output).
     *
     * @return list of children.
     */
    public List<Component> getChildren() {
        return children;
    }
}
