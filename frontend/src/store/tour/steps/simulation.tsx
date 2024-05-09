import { StepName, StepsConfiguration } from '@/store/tour';

export const simulationSteps: StepsConfiguration = {
    enumName: StepName.SIMULATIONSTEPS,
    list: [
        {
            selector: '.tour-step-0-simulation',
            content:
                "Welcome to simulation👋. ❗❗❗Please be sure to run this tutorial in a twin you created, this will prevent changing other peoples work. The tutorial will tell you when to use the → buttons, otherwise just click on the button it shows. Enough talk, let's get to it. Let's create a simulation",
        },
        {
            selector: '.tour-step-1-simulation',
            content:
                'Fill in the name, start time and date, end time and date and the timestep. (and click →)❗❗❗These fields are required to create a simulation. The timestep cannot be 0.',
        },
        {
            selector: '.tour-step-2-simulation',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-3-simulation',
            content: 'Choose the simulators you would like to use. (and click →)',
        },
        {
            selector: '.tour-step-4-simulation',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-5-simulation',
            content:
                'You can read more about these settings in the documentation. We will skip them for now.',
        },
        {
            selector: '.tour-step-6-simulation',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-7-simulation',
            content:
                'This window is identical to the editor. You can follow the editor tutorial to find out how to use it. (click →)',
        },
        {
            selector: '.tour-step-8-simulation',
            content: 'Let the magic happen!✨✨✨',
        },
        {
            selector: '.tour-step-9-simulation',
            content:
                'To delete simulations, use the checkboxes to select which ones you would like to delete',
        },
        {
            selector: '.tour-step-10-simulation',
            content: "Let's see an overview of the simulations that will be deleted",
        },
        {
            selector: '.tour-step-11-simulation',
            content: 'Delete the simulations',
        },
    ],
};
