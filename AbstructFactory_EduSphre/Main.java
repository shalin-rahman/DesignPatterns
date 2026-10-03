package AbstructFactory_EduSphre;

public class Main {
    public static void main(String[] args) {
        for (DeliveryMode mode : DeliveryMode.values()) {
            EduSphereCourse course = new EduSphereCourse(factoryFor(mode));
            course.runCourseWorkflow();
        }
    }

    // The only place that names a concrete factory. Adding a new mode means
    // a new enum value, a new factory class and one new case here;
    // EduSphereCourse stays as it is.
    static CourseComponentFactory factoryFor(DeliveryMode mode) {
        return switch (mode) {
            case ONLINE -> new OnlineCourseComponentFactory();
            case OFFLINE -> new OfflineCourseComponentFactory();
            default -> throw new IllegalArgumentException("Unknown delivery mode: " + mode);
        };
    }
}
