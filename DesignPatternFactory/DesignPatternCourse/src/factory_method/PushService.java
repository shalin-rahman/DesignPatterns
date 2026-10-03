package factory_method;

// NEW: added later - no existing file was edited

class PushService extends NotificationService {
    @Override
    protected Notification createNotification() {
        return new PushNotification();
    }
}

class SmsService extends NotificationService {
    @Override
    protected Notification createNotification() {
        return new SmsNotification();
    }
}
