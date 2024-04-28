import { Modal, ModalBody, ModalHeader, Label, Button, Textarea } from 'flowbite-react';
import { useEffect, useState, useRef } from 'react';
import Icon from '@mdi/react';
import { mdiSolarPanel, mdiWindPower, mdiGasBurner } from '@mdi/js';
import { icon } from 'leaflet';
import { BackendGetComponent, BackendGetSimulations } from '@/api/simulation/crud';
import { ComponentStructure } from '@/proto/simulation/simulation';
import { BackendCreatePreset } from '@/api/twins/crud';
import CustomJsonEditor, { TypeConverter } from '@/components/CustomJsonEditor';
import ToastNotification from '@/components/notification/ToastNotification';

interface CreateIconsModalProps {
    isModalOpen: boolean;
    closeModal: () => void;
    name?: string;
}

enum ModalPage {
    ICONS,
    INFOS,
}

function CreateIconsModal(propItems: CreateIconsModalProps) {
    const [modalPage, setModalPage] = useState<ModalPage>(ModalPage.ICONS);
    const basicFormRef = useRef<HTMLFormElement>(null);
    const [iCon, setIcon] = useState('');
    const [name, setName] = useState('');
    const [list_of_component_names, setListOfComponentNames] = useState<Map<string, string>>(
        new Map()
    );
    const [components, setComponents] = useState('{}');

    useEffect(() => {
        const getComponentResponse = async () => {
            try {
                const response = await BackendGetComponent();
                const components = response.components;
                for (let componentName in components) {
                    if (components.hasOwnProperty(componentName)) {
                        let componentSpec = components[componentName];
                        let componentStructure = componentSpec.structure;
                        if (componentSpec.type == 2) continue;
                        else if (componentName !== undefined) {
                            let item: { [key: string]: boolean | any } = {};
                            item[componentName] = TypeConverter(componentStructure);
                            list_of_component_names.set(componentName, JSON.stringify(item));
                        }
                    }
                }
            } catch (error) {
                console.error('Error fetching components:', error);
            }
        };

        getComponentResponse();
    }, []);

    const createPreset = async () => {
        try {
            if (name == '') {
                ToastNotification('warning', 'Preset must have a name');
                return;
            }

            let containsEdges = false;
            let containsNodes = false;
            Object.keys(JSON.parse(components)).map(item => {
                if (item.slice(-5) == '_edge') {
                    containsEdges = true;
                } else if (item.slice(-5) == '_node') {
                    containsNodes = true;
                }
            });

            if (containsEdges && containsNodes) {
                ToastNotification('warning', 'Cannot combine edges and notes in one preset');
                return;
            }

            //TODO change edge name for line for mapEditor 219, bad idea!!!
            await BackendCreatePreset(name.concat(containsEdges ? '_edge' : ''), components);
            handleCloseModal();
        } catch (error) {
            console.error('Error creating a preset', error);
        }
    };

    const handleNextButtonClick = async () => {
        switch (modalPage) {
            case ModalPage.ICONS: {
                setModalPage(modalPage + 1);
                break;
            }
            case ModalPage.INFOS: {
                await createPreset();
                break;
            }
        }
    };

    const reset = () => {
        setComponents('{}');
        -setName('');
        if (modalPage === ModalPage.INFOS) {
            setModalPage(modalPage - 1);
        }
    };

    const handlePreviousButtonClick = () => {
        if (modalPage === ModalPage.ICONS) {
            handleCloseModal();
        } else {
            setModalPage(modalPage - 1);
            reset();
        }
    };

    const handleCloseModal = () => {
        reset();
        propItems.closeModal();
    };

    useEffect(() => {
        console.log(JSON.parse(components));
    }, [components]);

    const handleInfo = (componentName: string) => {
        let tempComponents = JSON.parse(components);

        if (
            !tempComponents[componentName] &&
            list_of_component_names.get(componentName) &&
            componentName
        ) {
            tempComponents[componentName] = JSON.parse(
                list_of_component_names.get(componentName) || '{}'
            )[componentName];
        } else {
            delete tempComponents[componentName];
        }
        setComponents(JSON.stringify(tempComponents));
    };

    const handleChange = (e: string) => {
        setComponents(e);
    };

    return (
        <>
            <Modal
                show={propItems.isModalOpen}
                onClose={handleCloseModal}
                className='flex flex-row'
            >
                <ModalHeader>Create Icon</ModalHeader>
                <ModalBody>
                    {modalPage === ModalPage.ICONS && (
                        <div>
                            {list_of_component_names.size == 0 && (
                                <p>No simulators found with components</p>
                            )}
                            {Array.from(list_of_component_names.keys()).map((name, index) => (
                                <div key={name}>
                                    <Button
                                        style={{ marginBottom: '20px' }}
                                        outline={!JSON.parse(components)[name]}
                                        onClick={() => {
                                            handleInfo(name);
                                        }}
                                    >
                                        {name}
                                    </Button>
                                </div>
                            ))}
                        </div>
                    )}
                    {modalPage === ModalPage.INFOS && (
                        //producer information
                        <div className='my-4'>
                            <form ref={basicFormRef}>
                                <div>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='name' value='Preset name' />
                                    </div>
                                    <input
                                        id='name'
                                        className='bg-gray-50 border border-gray-300 text-gray-900 rounded-lg text-sm focus:ring-indigo-500 w-full focus:border-indigo-500 p-2.5'
                                        type='text'
                                        value={name}
                                        placeholder='name'
                                        required
                                        maxLength={50}
                                        onChange={e => setName(e.target.value)}
                                        style={{ marginBottom: '10px' }}
                                    />
                                </div>
                                <div>
                                    <div className='mb-2 block'>
                                        <Label htmlFor='gv' value='Variables' />
                                    </div>
                                    <CustomJsonEditor
                                        onSave={e => {
                                            handleChange(JSON.stringify(e));
                                        }}
                                        data={JSON.parse(components)}
                                    ></CustomJsonEditor>
                                </div>
                            </form>
                        </div>
                    )}
                </ModalBody>
                <Modal.Footer className='flex flex-row w-100'>
                    <Button
                        outline
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handlePreviousButtonClick}
                    >
                        {modalPage === ModalPage.ICONS ? 'Cancel' : 'Previous'}
                    </Button>
                    <div className='grow'></div>
                    <Button
                        color='indigo'
                        theme={{
                            color: {
                                indigo: 'bg-indigo-600 text-white ring-indigo-600',
                            },
                        }}
                        onClick={handleNextButtonClick}
                    >
                        {modalPage === ModalPage.INFOS ? 'Create' : 'Next'}
                    </Button>
                </Modal.Footer>
            </Modal>
        </>
    );
}

export default CreateIconsModal;
