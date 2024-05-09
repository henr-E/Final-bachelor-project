import { StepName, StepsConfiguration } from '@/store/tour';

export const editorSteps: StepsConfiguration = {
    enumName: StepName.EDITORSTEPS,
    list: [
        {
            selector: '.tour-step-0-editor',
            content:
                "Welcome to editor👋. ❗❗❗Please be sure to run this tutorial in a twin you created, this will prevent changing other peoples work. The tutorial will tell you when to use the → buttons, otherwise just click on the button it shows. Enough talk, let's get to it. Select a building (and click →)",
        },
        {
            selector: '.tour-step-1-editor',
            content: 'Delete the building',
        },
        {
            selector: '.tour-step-2-editor',
            content: 'Select the same building again to see the changes (and click →)',
        },
        {
            selector: '.tour-step-3-editor',
            content: 'Restore the building',
        },
        {
            selector: '.tour-step-4-editor',
            content:
                'We are going to create a sensor for a building. Make sure to select a building with no sensors. Buildings can only have one sensor so if you try to create another one you will get a warning. (and click →)',
            padding: { popover: [-10, 0, 0, 0] },
            //[top, right, bottom, left]
        },
        {
            selector: '.tour-step-5-editor',
            content: 'Create a sensor',
        },
        {
            selector: '.tour-step-6-editor',
            content:
                'Fill in a name and description (and click →) ❗❗❗These fields are required to create a sensor',
        },
        {
            selector: '.tour-step-7-editor',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-8-editor',
            content:
                'Here you can select a signal in the dropdown, then add it. Signals can also be deleted here using the X behind the signal name. (and click →)',
        },
        {
            selector: '.tour-step-9-editor',
            content: 'Go to the next step',
        },
        {
            selector: '.tour-step-10-editor',
            content: 'Change units, prefixes and aliases (and click →)',
        },
        {
            selector: '.tour-step-11-editor',
            content: 'Let the magic happen!✨✨✨',
        },
        {
            selector: '.tour-step-12-editor',
            content:
                'The created sensor will also be visible in the sensors tab. Select this sensor we just created to see the signals.',
        },
        {
            selector: '.tour-step-13-editor',
            content: 'lets go back to create some presets',
        },
        {
            selector: '.tour-step-14-editor',
            content: 'create preset',
        },
        {
            selector: '.tour-step-15-editor',
            content:
                "First, we will make a node and after that make an edge. Choose a node. (and click →) ❗❗❗You can't add a node and an edge to the same preset. This will give a warning",
        },
        {
            selector: '.tour-step-16-editor',
            content: 'Go to the next step.',
        },
        {
            selector: '.tour-step-17-editor',
            content:
                'Make a preset name (and click →)❗❗❗This name is required to create a preset',
        },
        {
            selector: '.tour-step-18-editor',
            content: 'Create the node preset.',
        },
        {
            selector: '.tour-step-19-editor',
            content: 'Add a new preset',
        },
        {
            selector: '.tour-step-20-editor',
            content:
                "Let's make an edge now. ❗❗❗You can't add a node and an edge to the same preset. This will give a warning",
        },
        {
            selector: '.tour-step-21-editor',
            content: 'Go to the next step.',
        },
        {
            selector: '.tour-step-22-editor',
            content: 'Make a preset name (and click →)',
        },
        {
            selector: '.tour-step-23-editor',
            content: 'Create the preset',
        },
        {
            selector: '.tour-step-24-editor',
            content:
                'To use a node preset, select the preset and then select a building. To use a transmission edge preset, select the preset and then select two buildings. Edges can only be applied to buildings that are nodes.',
        },
    ],
};
