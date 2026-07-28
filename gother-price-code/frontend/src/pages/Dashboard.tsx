import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Download, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/Dialog';
import { HotelGroupCard } from '@/components/HotelGroupCard';
import { ExcelUploader } from '@/components/ExcelUploader';
import {
  listHotelGroups,
  createHotelGroup,
  createHotelGroupWithExcel,
  deleteHotelGroup,
} from '@/api/hotelGroups';
import { downloadTemplate } from '@/api/hotels';
import { downloadBlob } from '@/api/scrapeJobs';

export function Dashboard() {
  const queryClient = useQueryClient();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [file, setFile] = useState<File | null>(null);

  // Fetch hotel groups
  const { data: groups, isLoading } = useQuery({
    queryKey: ['hotelGroups'],
    queryFn: listHotelGroups,
  });

  // Create mutation
  const createMutation = useMutation({
    mutationFn: async () => {
      if (file) {
        return createHotelGroupWithExcel(name, description || undefined, file);
      }
      return createHotelGroup({ name, description: description || undefined });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroups'] });
      setIsCreateOpen(false);
      resetForm();
    },
  });

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: deleteHotelGroup,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['hotelGroups'] });
    },
  });

  const resetForm = () => {
    setName('');
    setDescription('');
    setFile(null);
  };

  const handleCreate = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate();
  };

  const handleDownloadTemplate = async () => {
    const blob = await downloadTemplate();
    downloadBlob(blob, 'hotel-import-template.xlsx');
  };

  const handleDelete = (id: string) => {
    if (window.confirm('Are you sure you want to delete this hotel group?')) {
      deleteMutation.mutate(id);
    }
  };

  return (
    <div className="container mx-auto py-8 px-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold">Hotel Groups</h1>
          <p className="text-muted-foreground mt-1">
            Manage your hotel groups and run price comparisons
          </p>
        </div>
        <div className="flex items-center gap-3">
          <Button variant="outline" onClick={handleDownloadTemplate}>
            <Download className="h-4 w-4 mr-2" />
            Download Template
          </Button>
          <Dialog open={isCreateOpen} onOpenChange={setIsCreateOpen}>
            <DialogTrigger asChild>
              <Button>
                <Plus className="h-4 w-4 mr-2" />
                New Group
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <form onSubmit={handleCreate}>
                <DialogHeader>
                  <DialogTitle>Create Hotel Group</DialogTitle>
                  <DialogDescription>
                    Create a new group to organize hotels for price comparison
                  </DialogDescription>
                </DialogHeader>
                <div className="space-y-4 py-4">
                  <div className="space-y-2">
                    <Label htmlFor="name">Group Name *</Label>
                    <Input
                      id="name"
                      value={name}
                      onChange={(e) => setName(e.target.value)}
                      placeholder="e.g., Bangkok Hotels Q2"
                      required
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="description">Description</Label>
                    <Input
                      id="description"
                      value={description}
                      onChange={(e) => setDescription(e.target.value)}
                      placeholder="Optional description"
                    />
                  </div>
                  <div className="space-y-2">
                    <Label>Import Hotels (Optional)</Label>
                    <ExcelUploader
                      onFileSelect={setFile}
                      selectedFile={file}
                      onClear={() => setFile(null)}
                    />
                  </div>
                </div>
                <DialogFooter>
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setIsCreateOpen(false);
                      resetForm();
                    }}
                  >
                    Cancel
                  </Button>
                  <Button type="submit" disabled={createMutation.isPending}>
                    {createMutation.isPending ? (
                      <>
                        <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        Creating...
                      </>
                    ) : (
                      'Create Group'
                    )}
                  </Button>
                </DialogFooter>
              </form>
            </DialogContent>
          </Dialog>
        </div>
      </div>

      {/* Content */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : groups && groups.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {groups.map((group) => (
            <HotelGroupCard
              key={group.id}
              group={group}
              onDelete={handleDelete}
            />
          ))}
        </div>
      ) : (
        <div className="text-center py-12 border-2 border-dashed rounded-lg">
          <h3 className="text-lg font-semibold mb-2">No hotel groups yet</h3>
          <p className="text-muted-foreground mb-4">
            Create your first hotel group to start comparing prices
          </p>
          <Button onClick={() => setIsCreateOpen(true)}>
            <Plus className="h-4 w-4 mr-2" />
            Create Hotel Group
          </Button>
        </div>
      )}
    </div>
  );
}
