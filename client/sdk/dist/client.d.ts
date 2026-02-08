import { Template } from '../../../index';
export interface RenderOptions {
    template: string | Template;
    data?: any;
    output: string;
}
export declare class PDFCoreClient {
    constructor();
    render(options: RenderOptions): Promise<void>;
}
