import { StepName, StepsConfiguration } from '@/store/tour';

export const sensorsSteps: StepsConfiguration = {
    enumName: StepName.SENSORSSTEPS,
    list: [
        {
            selector: '.tour-step-0-sensors',
            content:
                "Welcome to sensors👋. ❗❗❗Please be sure to run this tutorial in a twin you created, this will prevent changing other peoples work. The tutorial will tell you when to use the → buttons, otherwise just click on the button it shows. Enough talk, let's get to it. Let's create a global sensor. The twin can only have one global sensor. If you try to create another one, you will get a warning.",
        },
        {
            selector: '.tour-step-1-sensors',
            content:
                'Fill in a name and description (and click →)❗❗❗These fields are required to create a sensor',
        },
        {
            selector: '.tour-step-2-sensors',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-3-sensors',
            content:
                'Here you can select a signal in the dropdown, then add it. Signals can also be deleted here using the X behind the signal name. (and click →)',
        },
        {
            selector: '.tour-step-4-sensors',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-5-sensors',
            content: 'Change units, prefixes and aliases (and click →)',
        },
        {
            selector: '.tour-step-6-sensors',
            content: 'Let the magic happen!✨✨✨',
        },
        {
            selector: '.tour-step-7-sensors',
            content: 'Select the created sensor',
        },
        {
            selector: '.tour-step-8-sensors',
            content: 'Update a sensor',
        },
        {
            selector: '.tour-step-9-sensors',
            content: 'Update the name and/or the description (and click →)',
        },
        {
            selector: '.tour-step-10-sensors',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-11-sensors',
            content: 'Add desired signals or delete signals (and click →)',
        },
        {
            selector: '.tour-step-12-sensors',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-13-sensors',
            content: 'Change units, prefixes and aliases (and click →)',
        },
        {
            selector: '.tour-step-14-sensors',
            content: 'Update the sensor',
        },
        {
            selector: '.tour-step-15-sensors',
            content: 'lets go back and delete sensors',
        },
        {
            selector: '.tour-step-16-sensors',
            content:
                'To delete sensors, use the checkboxes to select which ones you would like to delete',
        },
        {
            selector: '.tour-step-17-sensors',
            content: "Let's see an overview of the sensors that will be deleted",
        },
        {
            selector: '.tour-step-18-sensors',
            content: 'Delete the sensors',
        },
    ],
};
