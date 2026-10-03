package abstract_factory_2_cars;

class EuropeSpecification implements CarSpecification {
    public void display()
    {
        System.out.println(
            "Europe Car Specification: Fuel efficiency and emissions compliant with EU standards.");
    }
}
