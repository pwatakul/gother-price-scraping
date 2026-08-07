import { useCallback, useEffect, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import * as XLSX from 'xlsx';
import { Upload, FileSpreadsheet, X, Download } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { cn } from '@/utils';
import { downloadTemplate } from '@/api/hotels';
import { downloadBlob } from '@/api/scrapeJobs';

interface ExcelUploaderProps {
  onFileSelect: (file: File) => void;
  selectedFile: File | null;
  onClear: () => void;
}

const PREVIEW_ROW_COUNT = 3;

export function ExcelUploader({ onFileSelect, selectedFile, onClear }: ExcelUploaderProps) {
  const [isDownloading, setIsDownloading] = useState(false);
  const [previewRows, setPreviewRows] = useState<Record<string, unknown>[]>([]);
  const [totalRows, setTotalRows] = useState<number | null>(null);

  useEffect(() => {
    if (!selectedFile) {
      setPreviewRows([]);
      setTotalRows(null);
      return;
    }

    let cancelled = false;
    selectedFile.arrayBuffer().then((buf) => {
      if (cancelled) return;
      try {
        const workbook = XLSX.read(buf);
        const sheet = workbook.Sheets[workbook.SheetNames[0]];
        const rows = XLSX.utils.sheet_to_json<Record<string, unknown>>(sheet, { defval: null });
        setTotalRows(rows.length);
        setPreviewRows(rows.slice(0, PREVIEW_ROW_COUNT));
      } catch (e) {
        console.error('Failed to parse Excel file for preview:', e);
        setPreviewRows([]);
        setTotalRows(null);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [selectedFile]);

  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      if (acceptedFiles.length > 0) {
        onFileSelect(acceptedFiles[0]);
      }
    },
    [onFileSelect]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet': ['.xlsx'],
      'application/vnd.ms-excel': ['.xls'],
    },
    maxFiles: 1,
  });

  const handleDownloadTemplate = async () => {
    try {
      setIsDownloading(true);
      const blob = await downloadTemplate();
      downloadBlob(blob, 'hotel-import-template.xlsx');
    } catch (error) {
      console.error('Failed to download template:', error);
    } finally {
      setIsDownloading(false);
    }
  };

  if (selectedFile) {
    const columns = previewRows.length > 0 ? Object.keys(previewRows[0]) : [];

    return (
      <div className="space-y-3">
        <div className="flex items-center gap-3 p-3 border rounded-lg bg-green-50 border-green-200">
          <FileSpreadsheet className="h-6 w-6 text-green-600 shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium truncate text-green-800">
              ✅ {selectedFile.name} uploaded
              {totalRows !== null && ` / ${totalRows} row${totalRows !== 1 ? 's' : ''} detected`}
            </p>
          </div>
          <Button variant="ghost" size="icon" onClick={onClear}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        {columns.length > 0 && (
          <div className="border rounded-lg overflow-x-auto">
            <table className="w-full text-xs">
              <thead>
                <tr className="bg-[#f8fafc] border-b">
                  {columns.map((col) => (
                    <th key={col} className="px-3 py-2 text-left font-semibold uppercase tracking-wide text-slate-500">
                      {col}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {previewRows.map((row, i) => (
                  <tr key={i} className={cn('border-b last:border-0', i % 2 === 1 && 'bg-[#f8fafc]')}>
                    {columns.map((col) => {
                      const value = row[col];
                      const isBlank = value === null || value === undefined || value === '';
                      return (
                        <td key={col} className="px-3 py-2 whitespace-nowrap">
                          {isBlank ? (
                            <span className="italic text-slate-400">— (job default)</span>
                          ) : (
                            String(value)
                          )}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
            {totalRows !== null && totalRows > PREVIEW_ROW_COUNT && (
              <div className="px-3 py-1.5 text-xs text-muted-foreground bg-[#f8fafc] border-t">
                Showing {PREVIEW_ROW_COUNT} of {totalRows} rows
              </div>
            )}
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <div
        {...getRootProps()}
        className={cn(
          'border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors',
          isDragActive
            ? 'border-primary bg-primary/5'
            : 'border-muted-foreground/25 hover:border-primary/50'
        )}
      >
        <input {...getInputProps()} />
        <Upload className="h-10 w-10 mx-auto mb-3 text-muted-foreground" />
        {isDragActive ? (
          <p className="text-primary font-medium">Drop the Excel file here...</p>
        ) : (
          <>
            <p className="font-medium">Drag & drop an Excel file here</p>
            <p className="text-sm text-muted-foreground mt-1">
              or click to select a file (.xlsx, .xls)
            </p>
          </>
        )}
      </div>
      <div className="flex justify-center">
        <Button
          variant="outline"
          size="sm"
          onClick={handleDownloadTemplate}
          disabled={isDownloading}
        >
          <Download className="h-4 w-4 mr-2" />
          {isDownloading ? 'Downloading...' : 'Download Template'}
        </Button>
      </div>
    </div>
  );
}
