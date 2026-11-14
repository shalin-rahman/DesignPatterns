/**
 * DataSourceDecorator - the Base Decorator (Base Wrapper) class.
 *
 * Pattern Terminology:
 *   - WRAPPER / DECORATOR: An object that wraps around and decorates another object
 *              to augment its behavior dynamically. DataSourceDecorator and its
 *              subclasses act as wrappers/decorators.
 *   - WRAPPEE / DECORATED COMPONENT: The target object being enclosed/wrapped and
 *              decorated. It is stored in the {@code wrappee} field and also implements
 *              {@link DataSource}.
 *   - WRAP / DECORATE: The composition technique of enclosing a component within a
 *              decorator object to attach extra responsibilities without subclassing.
 *
 * This base wrapper/decorator does three essential things:
 *   1. Implements DataSource, matching the interface of its wrappee so clients
 *      cannot distinguish between the wrapper (decorator) and the wrappee (decorated object).
 *   2. Holds a reference to the wrappee/decorated target ({@code private final DataSource wrappee})
 *      via composition.
 *   3. Forwards all operations (writeData, readData) to the wrappee/decorated component by default,
 *      allowing concrete wrappers/decorators to override only what they specialize.
 *
 * In stacked chains:
 *   CompressionDecorator (Wrapper / Decorator)
 *     -> EncryptionDecorator (Wrappee/Decorated by Compression; Wrapper/Decorator to FileDataSource)
 *       -> FileDataSource (Innermost Wrappee / Decorated Core Component)
 */
public abstract class DataSourceDecorator implements DataSource {

    /**
     * The wrappee / decorated component: the target object being wrapped and decorated.
     * Can be the innermost concrete component (FileDataSource) or another wrapper/decorator.
     */
    private final DataSource wrappee;

    /**
     * Constructs a wrapper/decorator around the given wrappee/decorated component.
     *
     * @param source The wrappee / component instance to wrap and decorate
     */
    protected DataSourceDecorator(DataSource source) {
        this.wrappee = source;
    }

    // Default: Wrapper/Decorator delegates writeData directly to the wrappee / decorated component
    @Override
    public void writeData(String data) {
        wrappee.writeData(data);
    }

    // Default: Wrapper/Decorator delegates readData directly to the wrappee / decorated component
    @Override
    public String readData() {
        return wrappee.readData();
    }
}
