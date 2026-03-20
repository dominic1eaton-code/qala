import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-boards',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './boards.component.html',
  styleUrl: './boards.component.css'
})
export class SolutionsBoardsComponent {}
