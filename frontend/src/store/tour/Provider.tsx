import React, {
    createContext,
    Dispatch,
    ReactNode,
    SetStateAction,
    useCallback,
    useContext,
    useEffect,
    useState,
} from 'react';
import { useTour } from '@reactour/tour';

interface TourControlContextType {
    isOpen: Boolean;
    setCurrentStep: React.Dispatch<React.SetStateAction<number>>;
    customSetSteps: (stepsConfiguration: StepsConfiguration) => void;
    getCurrentStepsName: () => StepName | undefined;
    setIsOpen: Dispatch<SetStateAction<Boolean>>;
    nextStep: () => void;
    customGoToNextTourStep: (timeout: number) => void;
    customCloseTourAndStartAtStep: (step: number) => void;
}

enum StepName {
    STARTUPSTEPS,
    OVERVIEWSTEPS,
    EDITORSTEPS,
    REALTIMESTEPS,
    SENSORSSTEPS,
    SIMULATIONSTEPS,
}

interface StepDetail {
    selector: string;
    content: string;
    padding?: { popover?: number[] };
}

interface StepsConfiguration {
    enumName: StepName;
    list: StepDetail[];
}

export const useTourControl = (): TourControlContextType => {
    const { setIsOpen, isOpen, steps, setSteps, currentStep, setCurrentStep } = useTour();
    const [stepsName, setStepsName] = useState<StepName>();

    const nextStep = useCallback(() => {
        setCurrentStep(prev => prev + 1);
    }, [setCurrentStep]);

    /*
        The use of this function can be explained by giving an example
        When a modal is opened and the user clicks on next during the tour, the next modal that will open is rendered
        milliseconds after the tour renders its explanation window, therefor waiting some small time will help.
        Once the timout is over, the button is rendered and the tour can find this button and continue
     */
    const customGoToNextTourStep = useCallback(
        (timeout: number) => {
            if (steps.length > 0) {
                setTimeout(() => {
                    nextStep();
                }, timeout);
            }
        },
        [steps, nextStep]
    );

    /*
    The use of this function can be explained by giving an example
    There is a create twin button in the dropdown in the dashboardNavBar. When the tour shows this button to the user
    and the user clicks the button, the tour automatically goes to the next step becuase the Create Twin button dissapears
    The solution is to stop the tour, wait some time and continue the tour a the next stap (that is passed to this function)
     */
    const customCloseTourAndStartAtStep = useCallback(
        (step: number) => {
            if (steps.length != 0) {
                setIsOpen(false);
                setTimeout(() => {
                    setIsOpen(true);
                    setCurrentStep(step);
                }, 2000);
            }
        },
        [steps.length, setIsOpen, setCurrentStep]
    );

    const customSetSteps = useCallback(
        (stepsConfiguration: StepsConfiguration) => {
            if (setSteps) {
                setSteps(stepsConfiguration.list);
                setStepsName(stepsConfiguration.enumName);
            }
        },
        [setSteps]
    );

    const getCurrentStepsName = useCallback(() => {
        return stepsName;
    }, [stepsName]);

    return {
        isOpen,
        setCurrentStep,
        customSetSteps,
        getCurrentStepsName,
        setIsOpen,
        nextStep,
        customGoToNextTourStep,
        customCloseTourAndStartAtStep,
    };
};

const TourControlContext = createContext<TourControlContextType | undefined>(undefined);

function TourControlProvider({ children }: { children: React.ReactNode }) {
    const tourControl = useTourControl();

    return (
        <TourControlContext.Provider value={tourControl}>{children}</TourControlContext.Provider>
    );
}

export { type StepsConfiguration, StepName, TourControlProvider, TourControlContext };
