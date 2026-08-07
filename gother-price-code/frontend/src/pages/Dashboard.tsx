import { useMemo, useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Plus, Loader2, Search, Building2, Hotel as HotelIcon, Clock } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Label } from '@/components/ui/Label';
import { Card } from '@/components/ui/Card';
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
import { formatRelativeTime } from '@/utils/format';

export function Dashboard() {
  const queryClient = useQueryClient();
  const [isCreateOpen, setIsCreateOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [file, setFile] = useState<File | null>(null);
  const [search, setSearch] = useState('');

  // Fetch hotel groups
  const { data: groups, isLoading } = useQuery({
    queryKey: ['hotelGroups'],
    queryFn: listHotelGroups,
  });

  const filteredGroups = useMemo(() => {
    if (!groups) return groups;
    const q = search.trim().toLowerCase();
    if (!q) return groups;
    return groups.filter(
      (g) => g.name.toLowerCase().includes(q) || g.description?.toLowerCase().includes(q)
    );
  }, [groups, search]);

  // KPIs — only real, backed-by-data metrics. No fabricated win-rate or
  // parity-violation numbers (that's REQ-003, not built).
  const totalGroups = groups?.length ?? 0;
  const totalHotels = groups?.reduce((sum, g) => sum + g.hotel_count, 0) ?? 0;
  const lastScrapedAt = groups?.reduce<string | null>((latest, g) => {
    if (!g.last_scraped_at) return latest;
    if (!latest || g.last_scraped_at > latest) return g.last_scraped_at;
    return latest;
  }, null);

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

  const handleDelete = (id: string) => {
    if (window.confirm('Are you sure you want to delete this hotel group?')) {
      deleteMutation.mutate(id);
    }
  };

  return (
    <div className="max-w-[1400px] mx-auto py-6 px-7">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-xl font-bold">Hotel Groups</h1>
          <p className="text-muted-foreground mt-1 text-sm">
            Manage your hotel groups and run price comparisons
          </p>
        </div>
        <div className="flex items-center gap-3">
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

      {/* KPIs — only metrics backed by real data */}
      <div className="grid grid-cols-3 gap-3 mb-6">
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-sky-50 flex items-center justify-center">
              <Building2 className="h-4 w-4 text-sky-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none">{totalGroups}</div>
              <div className="text-xs text-muted-foreground mt-1">Total Groups</div>
            </div>
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-sky-50 flex items-center justify-center">
              <HotelIcon className="h-4 w-4 text-sky-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none">{totalHotels}</div>
              <div className="text-xs text-muted-foreground mt-1">Total Hotels</div>
            </div>
          </div>
        </Card>
        <Card className="p-4">
          <div className="flex items-center gap-3">
            <div className="h-9 w-9 rounded-full bg-sky-50 flex items-center justify-center">
              <Clock className="h-4 w-4 text-sky-600" />
            </div>
            <div>
              <div className="text-[26px] font-bold leading-none">
                {lastScrapedAt ? formatRelativeTime(lastScrapedAt) : 'Never'}
              </div>
              <div className="text-xs text-muted-foreground mt-1">Last Scraped</div>
            </div>
          </div>
        </Card>
      </div>

      {/* Search */}
      <div className="relative mb-4 max-w-sm">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Search hotel groups..."
          className="pl-9"
        />
      </div>

      {/* Content */}
      {isLoading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : filteredGroups && filteredGroups.length > 0 ? (
        <div className="grid gap-3 md:grid-cols-2">
          {filteredGroups.map((group) => (
            <HotelGroupCard
              key={group.id}
              group={group}
              onDelete={handleDelete}
            />
          ))}
          {!search && (
            <button
              type="button"
              onClick={() => setIsCreateOpen(true)}
              className="border-2 border-dashed border-slate-300 bg-[#f8fafc] rounded-[10px] flex items-center justify-center py-6 text-slate-400 hover:border-sky-400 hover:text-sky-600 transition-colors"
            >
              <div className="text-center">
                <div className="text-2xl mb-1">＋</div>
                <div className="text-sm font-semibold">Create New Group</div>
              </div>
            </button>
          )}
        </div>
      ) : search ? (
        <div className="text-center py-12 border-2 border-dashed rounded-lg">
          <p className="text-muted-foreground">No hotel groups match "{search}"</p>
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
