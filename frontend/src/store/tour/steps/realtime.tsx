import { StepName, StepsConfiguration } from '@/store/tour';

export const realtimeSteps: StepsConfiguration = {
    enumName: StepName.REALTIMESTEPS,
    list: [
        {
            selector: '.tour-step-0-realtime',
            content:
                "Welcome to realtime👋. Here you can see realtime data. You can only see real time data of sensors that were added to buildings. So if nothing appears here, you should first make some building sensors and a global sensor. To do this you can follow the sensors and editor tutorial. But enough talk, let's get to it. Here you can change the signal.",
        },
        {
            selector: '.tour-step-1-realtime',
            content: 'Choose the signal from the list',
        },
    ],
};
