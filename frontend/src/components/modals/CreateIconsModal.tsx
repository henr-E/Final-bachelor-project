import { Modal, ModalBody, ModalHeader, Label, Button, Table, Checkbox } from 'flowbite-react';
import { useEffect, useState, useRef } from 'react';
import { BackendGetComponent, BackendGetSimulations } from '@/api/simulation/crud';
import { BackendCreatePreset, BackendGetAllPreset } from '@/api/twins/crud';
import CustomJsonEditor, { TypeConverter } from '@/components/CustomJsonEditor';
import ToastNotification from '@/components/notification/ToastNotification';
import { ComponentSpecification } from '@/proto/simulation/simulation';

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
    const [allComponentSpec, setAllComponentSpec] = useState<Map<string, ComponentSpecification>>(
        new Map()
    );
    const [ComponentSpecSelected, setComponentSpecSelected] = useState<
        Map<string, ComponentSpecification>
    >(new Map());
    const [isValid, setIsValid] = useState(true);

    useEffect(() => {
        const getComponentResponse = async () => {
            /**
             * this function create a map with the structure
             * with the components from the simulation
             * {component_name: component_info}
             *
             * */
            try {
                const response = await BackendGetComponent();
                const components = response.components;
                let updatedComponentSelected = new Map<string, ComponentSpecification>(
                    allComponentSpec
                );
                for (let componentName in components) {
                    if (components.hasOwnProperty(componentName)) {
                        let componentSpec = components[componentName];
                        updatedComponentSelected.set(componentName, componentSpec);
                        let componentStructure = componentSpec.structure;
                        if (componentSpec.type == 2) continue;
                        else if (componentName !== undefined) {
                            let item: { [key: string]: boolean | any } = {};
                            item[componentName] = TypeConverter(componentStructure);
                            list_of_component_names.set(componentName, JSON.stringify(item));
                        }
                    }
                }
                setAllComponentSpec(updatedComponentSelected);
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
            const allPreset = await BackendGetAllPreset();

            if (allPreset !== undefined) {
                for (let i = 0; i < allPreset.length; i++) {
                    if (name === allPreset[i].name) {
                        ToastNotification('warning', 'preset' + name + ' already exist');
                        return;
                    }
                }
            }

            let containsEdges = false;
            for (let [key, item] of ComponentSpecSelected) {
                if (item.type == 1) {
                    containsEdges = true;
                }
            }

            await BackendCreatePreset(name, components, containsEdges);
            handleCloseModal();
        } catch (error) {
            console.error('Error creating a preset', error);
        }
    };

    const handleNextButtonClick = async () => {
        switch (modalPage) {
            case ModalPage.ICONS: {
                if (isValid) {
                    setModalPage(modalPage + 1);
                    break;
                } else return;
            }
            case ModalPage.INFOS: {
                await createPreset();
                break;
            }
        }
    };

    const reset = () => {
        setComponents('{}');
        ComponentSpecSelected.clear();
        setName('');
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
        let containsEdges = false;
        let containsNodes = false;
        for (let [name, spec] of ComponentSpecSelected) {
            if (spec.type === 1) {
                containsEdges = true;
            } else if (spec.type === 0) {
                containsNodes = true;
            }
        }
        setIsValid(true);
        if (containsEdges && containsNodes) {
            ToastNotification('warning', 'Cannot combine edges and notes in one preset');
            setIsValid(false);
            return;
        }
        if (ComponentSpecSelected.size == 0) {
            // if no component is selected
            setIsValid(false);
        }
    }, [components]);

    const handleInfo = (componentName: string) => {
        /**
         * this is will process the component_info for creating
         * a preset.
         */
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

    const handleAddingComponents = (name_: string, isChecked: boolean) => {
        for (let [name, spec] of allComponentSpec) {
            if (name === name_) {
                const updatedMap = new Map(ComponentSpecSelected);

                if (isChecked && spec !== undefined) {
                    // If checkbox is checked and spec is defined, add the component
                    updatedMap.set(name, spec);
                } else {
                    // If checkbox is unchecked, remove the component
                    updatedMap.delete(name);
                }
                // Update the state with the new map
                setComponentSpecSelected(updatedMap);
            }
        }
    };

    const handleChange = (e: string) => {
        setComponents(e);
    };

    const handleSetName = (name: string) => {
        setName(name);
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
                        <div className='overflow-x-auto'>
                            <Table hoverable>
                                <Table.Head>
                                    <Table.HeadCell className='p-4'></Table.HeadCell>
                                    <Table.HeadCell>COMPONENTS NAME</Table.HeadCell>
                                </Table.Head>
                                <Table.Body className='divide-y'>
                                    {list_of_component_names.size == 0 && (
                                        <p>No simulators found with components</p>
                                    )}
                                    {Array.from(list_of_component_names.keys()).map(
                                        (name, index) => (
                                            <Table.Row key={name}>
                                                <Table.Cell>
                                                    <Checkbox
                                                        id='checkbox'
                                                        checked={components.includes(name)}
                                                        onChange={event => {
                                                            const isChecked = event.target.checked;
                                                            handleInfo(name);
                                                            handleAddingComponents(name, isChecked);
                                                        }}
                                                        className='w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500 focus:ring-2 dark:bg-gray-700'
                                                    ></Checkbox>
                                                </Table.Cell>
                                                <Table.Cell>{name}</Table.Cell>
                                            </Table.Row>
                                        )
                                    )}
                                </Table.Body>
                            </Table>
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
                                        onChange={e => handleSetName(e.target.value)}
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
