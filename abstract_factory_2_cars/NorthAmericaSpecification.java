package abstract_factory_2_cars;

class NorthAmericaSpecification
    implements CarSpecification {
    public void display()
    {
        System.out.println(
            "North America Car Specification: Safety features compliant with local regulations.");
    }
}
