/**
 * PDFCore Editor — Shared Type Definitions
 *
 * These types are the canonical representation of PDFCore layout primitives.
 * They are used by the converter, PropertyInspector, and template IO.
 */

// ─── Color ──────────────────────────────────────────────────────────────────

export interface PDFColor {
  r: number; // 0-1
  g: number;
  b: number;
  a?: number;
}

// ─── Text Styles ────────────────────────────────────────────────────────────

export interface TextStyle {
  fontFamily?: string;
  fontSize?: number;
  fontWeight?: 'normal' | 'bold';
  fontStyle?: 'normal' | 'italic';
  color?: PDFColor;
  backgroundColor?: PDFColor;
  textAlign?: 'left' | 'center' | 'right' | 'justify';
  lineHeight?: number;
  letterSpacing?: number;
  rotation?: number;
  opacity?: number;
  borderWidth?: number;
  borderColor?: PDFColor;
  padding?: number;
}

// ─── Image Styles ───────────────────────────────────────────────────────────

export interface ImageStyle {
  width?: number;
  height?: number;
  opacity?: number;
  borderWidth?: number;
  borderColor?: PDFColor;
  objectFit?: 'contain' | 'cover' | 'fill';
  rotation?: number;
}

// ─── Shape Styles ───────────────────────────────────────────────────────────

export interface ShapeStyle {
  width?: number;
  height?: number;
  fill?: PDFColor;
  strokeWidth?: number;
  strokeColor?: PDFColor;
  opacity?: number;
  rotation?: number;
}

// ─── Table Styles ───────────────────────────────────────────────────────────

export interface TableStyle {
  headerBg?: string;
  headerColor?: string;
  borderColor?: string;
  cellPadding?: number;
  fontSize?: number;
  striped?: boolean;
  alternateRowColor?: string;
}

export interface TableColumn {
  header: string;
  width?: number;
  align?: 'left' | 'center' | 'right';
}

// ─── Template Node ──────────────────────────────────────────────────────────

export interface TemplateNode {
  type: string;
  [key: string]: unknown;
}

// ─── Page Settings ──────────────────────────────────────────────────────────

export interface PageSettings {
  size: 'A4' | 'Letter' | 'Legal' | 'Custom';
  orientation: 'portrait' | 'landscape';
  width?: number;   // mm, for Custom
  height?: number;  // mm, for Custom
  margins: { top: number; bottom: number; left: number; right: number };
  backgroundColor?: string;
}

// ─── Template Manifest ──────────────────────────────────────────────────────

export interface ManifestInfo {
  name?: string;
  version?: string;
  author?: string;
  engineVersion?: string;
  createdAt?: string;
  modifiedAt?: string;
}

// ─── Template Package ───────────────────────────────────────────────────────

export interface PDFCoreTemplate {
  root: TemplateNode;
  pages?: TemplateNode[];   // multi-page support
  manifest?: ManifestInfo;
  styles: Record<string, unknown>;
  settings?: PageSettings;
  queries?: QueryDefinition[];
}

// ─── Data Binding ───────────────────────────────────────────────────────────

export interface QueryDefinition {
  name: string;
  sql: string;
  params?: string[];
  description?: string;
}

export interface DataBinding {
  queryName: string;
  field: string;
  format?: string; // e.g., "{value}" or "$ {value}"
}

// ─── Editor State ───────────────────────────────────────────────────────────

export interface EditorPage {
  id: string;
  label: string;
  settings?: Partial<PageSettings>;
}

export interface EditorState {
  pages: EditorPage[];
  activePageIndex: number;
  zoom: number;
  showGrid: boolean;
  snapToGrid: boolean;
  gridSize: number;
}

// ─── Page Dimensions (in px for canvas) ─────────────────────────────────────

export const PAGE_SIZES: Record<string, { width: number; height: number }> = {
  'A4': { width: 595, height: 842 },       // 210mm x 297mm at 72dpi
  'Letter': { width: 612, height: 792 },   // 8.5" x 11" at 72dpi
  'Legal': { width: 612, height: 1008 },   // 8.5" x 14" at 72dpi
};

// ─── Block Categories (spec §25.3) ──────────────────────────────────────────

export const BLOCK_CATEGORIES = {
  Layout: ['Section', 'Row', 'Column', 'Container'],
  Text: ['Paragraph', 'Heading', 'Label'],
  Data: ['Dynamic Text', 'Table', 'Repeater'],
  Media: ['Image', 'Background Image'],
  Shapes: ['Rectangle', 'Circle', 'Line'],
  Utility: ['Page Break', 'Header', 'Footer', 'Page Number'],
  Navigation: ['Bookmark', 'Hyperlink'],
} as const;
