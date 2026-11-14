/**
 * DataSourceDecorator - the base class all decorators extend.
 *
 * This is the core of the Decorator pattern. It does three things:
 *   1. Implements DataSource, so it can be used anywhere a DataSource is expected.
 *   2. Holds a reference to another DataSource (the wrappee) via composition.
 *   3. By default, forwards every call to the wrappee unchanged.
 *
 * Concrete decorators (EncryptionDecorator, CompressionDecorator) extend this
 * and only override the methods where they add behaviour. Everything else
 * passes through automatically without any extra code in the subclass.
 *
 * Example chain:
 *   CompressionDecorator -> EncryptionDecorator -> FileDataSource
 *
 * A call to writeData() on the outer wrapper flows inward step by step
 * until it reaches FileDataSource, which does the actual disk write.
 */
public abstract class DataSourceDecorator implements DataSource {

    // The object being wrapped — could be a FileDataSource or another decorator
    private final DataSource wrappee;

    protected DataSourceDecorator(DataSource source) {
        this.wrappee = source;
    }

    // Default: forward the call to the next layer, unchanged
    @Override
    public void writeData(String data) {
        wrappee.writeData(data);
    }

    // Default: forward the call to the next layer, unchanged
    @Override
    public String readData() {
        return wrappee.readData();
    }
}
