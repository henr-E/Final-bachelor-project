import { StepName, StepsConfiguration } from '@/store/tour';

export const overviewSteps: StepsConfiguration = {
    enumName: StepName.OVERVIEWSTEPS,
    list: [
        {
            selector: '.tour-step-0-overview',
            content:
                "Welcome to overview👋. The tutorial will tell you when to use the → buttons, otherwise just click on the button it shows. Enough talk, let's get to it. Here you can switch to a different twin or create a twin",
        },
        {
            selector: '.tour-step-1-overview',
            content: "Let's create a twin!",
        },
        {
            selector: '.tour-step-2-overview',
            content:
                "Choose the city, place or choose coordinates (and click →). (you don't have to choose coordinates if you choose a place and vice versa.) ❗❗❗These fields are required to create a twin",
        },
        {
            selector: '.tour-step-3-overview',
            content: 'Search the city or place',
        },
        {
            selector: '.tour-step-4-overview',
            content:
                'Change the radius. You can choose a radius between 400 meters and 1000 meters (and click →)',
        },
        {
            selector: '.tour-step-5-overview',
            content: 'Enter a custom name if you want to (and click →)',
        },
        {
            selector: '.tour-step-6-overview',
            content: "Let's see an overview",
        },
        {
            selector: '.tour-step-7-overview',
            content: 'Check if the settings are correct and let the magic happen!',
        },
        {
            selector: '.tour-step-8-overview',
            content:
                'To delete twins, use the checkboxes to select which ones you would like to delete',
        },
        {
            selector: '.tour-step-9-overview',
            content: "Let's see an overview of the twins that will be deleted",
        },
        {
            selector: '.tour-step-10-overview',
            content: 'Delete the twins',
        },
    ],
};
